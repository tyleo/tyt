use crate::{ByteReader, Result, invalid};
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use std::iter;

/// The run marker in `.qb` RLE data: the next two `u32`s are a count and color.
const CODE_FLAG: u32 = 2;

/// The end-of-slice marker in `.qb` RLE data.
const NEXT_SLICE_FLAG: u32 = 6;

/// Parses a Qubicle Binary `.qb` file into a [`QbFile`].
///
/// Reads the header flags, then each matrix's name, size, position, and voxel
/// grid. RLE and raw voxel data both decode to the same dense grid. Parsing is
/// bounds-checked, so a truncated or malformed file is rejected rather than
/// masked.
pub fn from_qb_file_bytes(bytes: &[u8]) -> Result<QbFile> {
    let mut reader = ByteReader::new(bytes);

    let version = reader.read_u32()?;
    let color_format = read_color_format(reader.read_u32()?)?;
    let z_axis_orientation = read_z_axis_orientation(reader.read_u32()?)?;
    let compressed = reader.read_u32()? != 0;
    let visibility_mask_encoded = reader.read_u32()? != 0;
    let matrix_count = reader.read_u32()? as usize;

    let mut matrices = Vec::with_capacity(matrix_count.min(reader.remaining().len() / 16));
    for _ in 0..matrix_count {
        matrices.push(read_matrix(&mut reader, color_format, compressed)?);
    }

    if !reader.is_empty() {
        return Err(invalid(format!(
            "{} unexpected trailing bytes after the last matrix",
            reader.remaining().len()
        )));
    }

    Ok(QbFile {
        version,
        color_format,
        z_axis_orientation,
        compressed,
        visibility_mask_encoded,
        matrices,
    })
}

/// Maps the header `colorFormat` field to its variant.
fn read_color_format(value: u32) -> Result<QbColorFormat> {
    match value {
        0 => Ok(QbColorFormat::Rgba),
        1 => Ok(QbColorFormat::Bgra),
        other => Err(invalid(format!(
            "colorFormat is {other}, expected 0 (RGBA) or 1 (BGRA)"
        ))),
    }
}

/// Maps the header `zAxisOrientation` field to its variant.
fn read_z_axis_orientation(value: u32) -> Result<QbZAxisOrientation> {
    match value {
        0 => Ok(QbZAxisOrientation::LeftHanded),
        1 => Ok(QbZAxisOrientation::RightHanded),
        other => Err(invalid(format!(
            "zAxisOrientation is {other}, expected 0 (left handed) or 1 (right handed)"
        ))),
    }
}

/// Reads one matrix header and its voxel grid.
fn read_matrix(
    reader: &mut ByteReader,
    color_format: QbColorFormat,
    compressed: bool,
) -> Result<QbMatrix> {
    let name_len = reader.read_u8()? as usize;
    let name = reader.read_string(name_len)?;
    let size = [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?];
    let position = [reader.read_i32()?, reader.read_i32()?, reader.read_i32()?];

    let voxels = if compressed {
        read_compressed_voxels(reader, size, color_format)?
    } else {
        read_raw_voxels(reader, size, color_format)?
    };

    Ok(QbMatrix {
        name,
        size,
        position,
        voxels,
    })
}

/// The number of cells in a `size`, or an error if it overflows `usize`.
fn voxel_count(size: [u32; 3]) -> Result<usize> {
    (size[0] as usize)
        .checked_mul(size[1] as usize)
        .and_then(|xy| xy.checked_mul(size[2] as usize))
        .ok_or_else(|| {
            invalid(format!(
                "matrix size {size:?} overflows the addressable range"
            ))
        })
}

/// Reads a raw (uncompressed) voxel grid: one four-byte color per cell, in
/// `Z` / `Y` / `X` order. Read in one bounds-checked block, so a size larger
/// than the remaining bytes is rejected.
fn read_raw_voxels(
    reader: &mut ByteReader,
    size: [u32; 3],
    color_format: QbColorFormat,
) -> Result<Vec<QbVoxel>> {
    let count = voxel_count(size)?;
    let byte_len = count.checked_mul(4).ok_or_else(|| {
        invalid(format!(
            "matrix size {size:?} overflows the addressable range"
        ))
    })?;
    Ok(reader
        .read_bytes(byte_len)?
        .chunks_exact(4)
        .map(|cell| decode_color([cell[0], cell[1], cell[2], cell[3]], color_format))
        .collect())
}

/// Reads a run-length-encoded voxel grid, decoding each `Z` slice's runs into
/// the dense grid. Cells fill in index order (`x` innermost); a slice short on
/// run data is padded with empty cells.
fn read_compressed_voxels(
    reader: &mut ByteReader,
    size: [u32; 3],
    color_format: QbColorFormat,
) -> Result<Vec<QbVoxel>> {
    let [size_x, size_y, size_z] = size.map(|value| value as usize);
    let slice_len = size_x.checked_mul(size_y).ok_or_else(|| {
        invalid(format!(
            "matrix size {size:?} overflows the addressable range"
        ))
    })?;
    let total = voxel_count(size)?;

    let mut voxels = Vec::new();
    for _ in 0..size_z {
        let slice_start = voxels.len();
        loop {
            let data = reader.read_u32()?;
            if data == NEXT_SLICE_FLAG {
                break;
            }
            let filled = voxels.len() - slice_start;
            if data == CODE_FLAG {
                let count = reader.read_u32()? as usize;
                let color = reader.read_u32()?;
                if count > slice_len - filled {
                    return Err(invalid(format!(
                        "RLE run of {count} voxels overruns the {slice_len}-cell slice"
                    )));
                }
                let voxel = decode_color(color.to_le_bytes(), color_format);
                voxels.extend(iter::repeat_n(voxel, count));
            } else {
                if filled >= slice_len {
                    return Err(invalid(format!(
                        "RLE slice holds more than its {slice_len} cells"
                    )));
                }
                voxels.push(decode_color(data.to_le_bytes(), color_format));
            }
        }
        // A slice that stops short of its cells leaves the rest empty.
        voxels.resize(slice_start + slice_len, QbVoxel::default());
    }
    debug_assert_eq!(voxels.len(), total);

    Ok(voxels)
}

/// Decodes four stored bytes into a voxel, undoing a `BGRA` channel order.
fn decode_color(bytes: [u8; 4], color_format: QbColorFormat) -> QbVoxel {
    let [b0, b1, b2, visibility] = bytes;
    match color_format {
        QbColorFormat::Rgba => QbVoxel {
            r: b0,
            g: b1,
            b: b2,
            visibility,
        },
        QbColorFormat::Bgra => QbVoxel {
            r: b2,
            g: b1,
            b: b0,
            visibility,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::qb::{from_qb_file_bytes, to_qb_file_bytes};
    use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};

    /// Two matrices exercising an RLE run, an empty cell, a multi-slice matrix,
    /// and a cell whose stored `u32` collides with [`CODE_FLAG`], parameterized
    /// by the storage flags.
    fn sample_file(compressed: bool, color_format: QbColorFormat) -> QbFile {
        let a = QbVoxel::new(200, 10, 10);
        let b = QbVoxel::new(10, 200, 10);
        // An empty cell whose stored bytes are [2, 0, 0, 0] -> u32 2 == CODE_FLAG,
        // which the RLE writer must escape so it round-trips.
        let flaggy = QbVoxel {
            r: 2,
            g: 0,
            b: 0,
            visibility: 0,
        };
        QbFile {
            version: 257,
            color_format,
            z_axis_orientation: QbZAxisOrientation::RightHanded,
            compressed,
            visibility_mask_encoded: false,
            matrices: vec![
                QbMatrix {
                    name: "first".to_owned(),
                    size: [2, 2, 1],
                    position: [1, -2, 3],
                    // index = x + 2*(y + 2*z): a run of two `a`, then `b`, then `flaggy`.
                    voxels: vec![a, a, b, flaggy],
                },
                QbMatrix {
                    name: "second".to_owned(),
                    size: [1, 1, 2],
                    position: [0, 0, 0],
                    voxels: vec![QbVoxel::default(), a],
                },
            ],
        }
    }

    #[test]
    fn round_trips_uncompressed() {
        let file = sample_file(false, QbColorFormat::Rgba);
        let bytes = to_qb_file_bytes(&file);
        assert_eq!(from_qb_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn round_trips_compressed() {
        let file = sample_file(true, QbColorFormat::Rgba);
        let bytes = to_qb_file_bytes(&file);
        assert_eq!(from_qb_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn round_trips_bgra() {
        let file = sample_file(true, QbColorFormat::Bgra);
        let bytes = to_qb_file_bytes(&file);
        assert_eq!(from_qb_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn round_trips_empty_file() {
        let file = QbFile::default();
        let bytes = to_qb_file_bytes(&file);
        assert_eq!(from_qb_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn parses_hand_written_uncompressed() {
        // version 257, RGBA, left-handed, uncompressed, no mask, one 1x1x1 matrix.
        let mut bytes = Vec::new();
        for value in [257u32, 0, 0, 0, 0, 1] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.push(1);
        bytes.extend_from_slice(b"m");
        for value in [1u32, 1, 1] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [0i32, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&[7, 8, 9, 255]);

        let file = from_qb_file_bytes(&bytes).unwrap();
        assert_eq!(file.matrices.len(), 1);
        assert_eq!(file.matrices[0].name, "m");
        assert_eq!(file.matrices[0].size, [1, 1, 1]);
        assert_eq!(file.matrices[0].voxels, vec![QbVoxel::new(7, 8, 9)]);
        assert_eq!(file.matrices[0].voxel(0, 0, 0), Some(QbVoxel::new(7, 8, 9)));
    }

    #[test]
    fn rejects_unknown_color_format() {
        let mut bytes = Vec::new();
        for value in [257u32, 9, 0, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        assert!(from_qb_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = to_qb_file_bytes(&QbFile::default());
        bytes.push(0);
        assert!(from_qb_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated() {
        assert!(from_qb_file_bytes(b"").is_err());
        assert!(from_qb_file_bytes(&[0, 0, 0]).is_err());
    }
}
