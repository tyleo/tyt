use crate::ByteWriter;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};

/// The run marker in `.qb` RLE data: the next two `u32`s are a count and color.
const CODE_FLAG: u32 = 2;

/// The end-of-slice marker in `.qb` RLE data.
const NEXT_SLICE_FLAG: u32 = 6;

/// Serializes a [`QbFile`] to a Qubicle Binary `.qb` file, the inverse of
/// [`from_qb_file_bytes`](crate::qb::from_qb_file_bytes).
///
/// Writes the header flags, then each matrix's name, size, position, and voxel
/// grid. Voxel data is run-length encoded when the file's
/// [`compressed`](QbFile::compressed) flag is set, raw otherwise. The RLE form
/// may differ byte-for-byte from another encoder's (run boundaries are an
/// encoder choice) but decodes to the same grid, so a decoded file re-encodes
/// to an equivalent one.
pub fn to_qb_file_bytes(file: &QbFile) -> Vec<u8> {
    let mut out = ByteWriter::new();

    out.write_u32(file.version);
    out.write_u32(match file.color_format {
        QbColorFormat::Rgba => 0,
        QbColorFormat::Bgra => 1,
    });
    out.write_u32(match file.z_axis_orientation {
        QbZAxisOrientation::LeftHanded => 0,
        QbZAxisOrientation::RightHanded => 1,
    });
    out.write_u32(file.compressed as u32);
    out.write_u32(file.visibility_mask_encoded as u32);
    out.write_len(file.matrices.len());

    for matrix in &file.matrices {
        write_matrix(&mut out, matrix, file.color_format, file.compressed);
    }

    out.into_bytes()
}

/// Writes one matrix header and its voxel grid.
fn write_matrix(
    out: &mut ByteWriter,
    matrix: &QbMatrix,
    color_format: QbColorFormat,
    compressed: bool,
) {
    let name = matrix.name.as_bytes();
    debug_assert!(
        name.len() <= u8::MAX as usize,
        "matrix name is {} bytes; the .qb format stores its length in a u8",
        name.len()
    );
    out.write_u8(name.len() as u8);
    out.write_bytes(name);
    for value in matrix.size {
        out.write_u32(value);
    }
    for value in matrix.position {
        out.write_i32(value);
    }

    if compressed {
        write_compressed_voxels(out, matrix, color_format);
    } else {
        for &voxel in &matrix.voxels {
            out.write_bytes(&encode_color(voxel, color_format));
        }
    }
}

/// Writes the voxel grid run-length encoded, one `Z` slice at a time. A run of
/// equal cells becomes a [`CODE_FLAG`] marker; a lone cell is written directly
/// unless its color collides with a flag value, in which case it too is escaped
/// as a one-cell run so it round-trips.
fn write_compressed_voxels(out: &mut ByteWriter, matrix: &QbMatrix, color_format: QbColorFormat) {
    let [size_x, size_y, size_z] = matrix.size.map(|value| value as usize);
    let slice_len = size_x.saturating_mul(size_y);

    for z in 0..size_z {
        let base = z.saturating_mul(slice_len);
        let slice = matrix
            .voxels
            .get(base..base.saturating_add(slice_len))
            .unwrap_or(&[]);

        let mut i = 0;
        while i < slice.len() {
            let color_bytes = encode_color(slice[i], color_format);
            let mut run = 1;
            while i + run < slice.len() && encode_color(slice[i + run], color_format) == color_bytes
            {
                run += 1;
            }
            let color = u32::from_le_bytes(color_bytes);
            if run > 1 {
                out.write_u32(CODE_FLAG);
                out.write_len(run);
                out.write_u32(color);
            } else if color == CODE_FLAG || color == NEXT_SLICE_FLAG {
                out.write_u32(CODE_FLAG);
                out.write_u32(1);
                out.write_u32(color);
            } else {
                out.write_u32(color);
            }
            i += run;
        }

        out.write_u32(NEXT_SLICE_FLAG);
    }
}

/// Encodes a voxel into its four stored bytes, applying a `BGRA` channel order.
fn encode_color(voxel: QbVoxel, color_format: QbColorFormat) -> [u8; 4] {
    match color_format {
        QbColorFormat::Rgba => [voxel.r, voxel.g, voxel.b, voxel.visibility],
        QbColorFormat::Bgra => [voxel.b, voxel.g, voxel.r, voxel.visibility],
    }
}
