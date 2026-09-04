use crate::{ByteReader, DecompressZlib, Error, Result, invalid};
use qbcl::qbcl::{
    QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};
use std::iter;

/// The matrix node type id.
const NODE_MATRIX: u32 = 0;

/// The model node type id.
const NODE_MODEL: u32 = 1;

/// The compound node type id.
const NODE_COMPOUND: u32 = 2;

/// The file-format version this crate reads and writes.
const FILE_VERSION: u32 = 2;

/// The RLE marker: a stream integer whose mask byte (its high byte) is this is a
/// run header, and its low byte is the run length.
const RLE_MASK: u8 = 2;

/// The deepest the reader will descend, so a pathologically nested file is
/// rejected rather than overflowing the stack.
const MAX_DEPTH: usize = 4096;

/// Parses a Qubicle Construction Library `.qbcl` file into a [`QbclFile`]
/// through `dependencies`.
///
/// Reads the header, preview thumbnail, the seven metadata strings, and the
/// scene tree in turn, zlib-decompressing and run-length-decoding each matrix's
/// voxel grid. Parsing is bounds-checked, so a truncated or malformed file is
/// rejected rather than masked. An unknown node type id is rejected, since the
/// format gives no per-node length with which to skip it.
pub fn from_qbcl_file_bytes<D: DecompressZlib>(dependencies: &D, bytes: &[u8]) -> Result<QbclFile> {
    let mut reader = ByteReader::new(bytes);

    let magic = reader.read_array::<4>()?;
    if &magic != b"QBCL" {
        return Err(invalid(format!(
            "not a .qbcl file: expected magic \"QBCL\", found {magic:?}"
        )));
    }
    let program_version = reader.read_u32()?;
    let file_version = reader.read_u32()?;
    if file_version != FILE_VERSION {
        return Err(invalid(format!(
            "unsupported .qbcl file version {file_version}, expected {FILE_VERSION}"
        )));
    }

    let thumbnail = read_thumbnail(&mut reader)?;
    let metadata = read_metadata(&mut reader)?;
    let guid = reader.read_array::<16>()?;
    let root = read_node(dependencies, &mut reader, 0)?;

    if !reader.is_empty() {
        return Err(invalid(format!(
            "{} unexpected trailing bytes after the scene tree",
            reader.remaining().len()
        )));
    }

    Ok(QbclFile {
        program_version,
        file_version,
        thumbnail,
        metadata,
        guid,
        root,
    })
}

/// Reads the preview thumbnail: its dimensions then its `BGRA` pixels, normalized
/// to [`QbclColor`]'s `RGBA`.
fn read_thumbnail(reader: &mut ByteReader) -> Result<QbclThumbnail> {
    let width = reader.read_u32()?;
    let height = reader.read_u32()?;
    let byte_len = pixel_count(width, height)?.checked_mul(4).ok_or_else(|| {
        invalid(format!(
            "thumbnail {width}x{height} overflows the byte range"
        ))
    })?;
    let pixels = reader
        .read_bytes(byte_len)?
        .chunks_exact(4)
        .map(|pixel| QbclColor::new(pixel[2], pixel[1], pixel[0], pixel[3]))
        .collect();
    Ok(QbclThumbnail {
        width,
        height,
        pixels,
    })
}

/// Reads the seven metadata strings, in the order the editor's UI lists them.
fn read_metadata(reader: &mut ByteReader) -> Result<QbclMetadata> {
    Ok(QbclMetadata {
        title: read_len_string(reader)?,
        description: read_len_string(reader)?,
        tags: read_len_string(reader)?,
        author: read_len_string(reader)?,
        company: read_len_string(reader)?,
        website: read_len_string(reader)?,
        copyright: read_len_string(reader)?,
    })
}

/// Reads a `u32`-length-prefixed UTF-8 string.
fn read_len_string(reader: &mut ByteReader) -> Result<String> {
    let len = reader.read_u32()? as usize;
    reader.read_string(len)
}

/// Reads one node: the common header (type, name, and editor flags) then a
/// type-specific body.
fn read_node<D: DecompressZlib>(
    dependencies: &D,
    reader: &mut ByteReader,
    depth: usize,
) -> Result<QbclNode> {
    if depth > MAX_DEPTH {
        return Err(invalid(format!(
            "scene tree is nested deeper than {MAX_DEPTH} nodes"
        )));
    }
    let type_id = reader.read_u32()?;
    let _unknown = reader.read_u32()?; // Always 1 in reference files; not modeled.
    let name = read_len_string(reader)?;
    let [visible, _flag, locked] = reader.read_array::<3>()?; // Middle flag always 1.

    let body = match type_id {
        NODE_MATRIX => QbclNodeBody::Matrix(read_matrix(dependencies, reader)?),
        NODE_MODEL => QbclNodeBody::Model(read_model(dependencies, reader, depth)?),
        NODE_COMPOUND => QbclNodeBody::Compound(read_compound(dependencies, reader, depth)?),
        other => return Err(invalid(format!("unknown .qbcl node type id {other}"))),
    };

    Ok(QbclNode {
        name,
        visible: visible != 0,
        locked: locked != 0,
        body,
    })
}

/// Reads a matrix body: size, position, pivot, then the compressed voxel grid.
fn read_matrix<D: DecompressZlib>(dependencies: &D, reader: &mut ByteReader) -> Result<QbclMatrix> {
    let size = [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?];
    let position = [reader.read_i32()?, reader.read_i32()?, reader.read_i32()?];
    let pivot = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];
    let data_size = reader.read_u32()? as usize;
    let voxels = read_voxels(dependencies, size, reader.read_bytes(data_size)?)?;
    Ok(QbclMatrix {
        size,
        position,
        pivot,
        voxels,
    })
}

/// Reads a model body: the 36-byte transform chunk then the child nodes.
fn read_model<D: DecompressZlib>(
    dependencies: &D,
    reader: &mut ByteReader,
    depth: usize,
) -> Result<QbclModel> {
    let transform = reader.read_array::<36>()?;
    let children = read_children(dependencies, reader, depth)?;
    Ok(QbclModel {
        transform,
        children,
    })
}

/// Reads a compound body: a matrix grid then the child nodes.
fn read_compound<D: DecompressZlib>(
    dependencies: &D,
    reader: &mut ByteReader,
    depth: usize,
) -> Result<QbclCompound> {
    let matrix = read_matrix(dependencies, reader)?;
    let children = read_children(dependencies, reader, depth)?;
    Ok(QbclCompound { matrix, children })
}

/// Reads a `u32` child count then that many child nodes, each one level deeper.
fn read_children<D: DecompressZlib>(
    dependencies: &D,
    reader: &mut ByteReader,
    depth: usize,
) -> Result<Vec<QbclNode>> {
    let count = reader.read_u32()? as usize;
    // A node header is at least 15 bytes (type, unknown, a zero-length name, and
    // three flag bytes), so a count larger than the remaining bytes allow is
    // malformed.
    let mut children = Vec::with_capacity(count.min(reader.remaining().len() / 15));
    for _ in 0..count {
        children.push(read_node(dependencies, reader, depth + 1)?);
    }
    Ok(children)
}

/// Decompresses and run-length-decodes a matrix's voxel grid into its dense
/// cells, in storage order. The grid is stored column by column (one column per
/// `(x, z)`, `x` outermost), each column a run of `size[1]` cells along `y`.
fn read_voxels<D: DecompressZlib>(
    dependencies: &D,
    size: [u32; 3],
    compressed: &[u8],
) -> Result<Vec<QbclVoxel>> {
    let [size_x, size_y, size_z] = size.map(|value| value as usize);
    let total = voxel_count(size)?;
    let columns = size_x.checked_mul(size_z).ok_or_else(|| {
        invalid(format!(
            "matrix size {size:?} overflows the addressable range"
        ))
    })?;

    let raw = dependencies
        .decompress_zlib(compressed)
        .map_err(Error::Zlib)?;
    let mut inner = ByteReader::new(&raw);

    let mut voxels = Vec::with_capacity(total);
    for _ in 0..columns {
        let mut produced = 0;
        let data_num = inner.read_u16()? as usize;
        let mut read = 0;
        while read < data_num {
            let value = inner.read_u32()?;
            read += 1;
            let [r, g, b, mask] = value.to_le_bytes();
            if mask == RLE_MASK {
                // A run: the low byte is the length, the next integer the color.
                let run = r as usize;
                let color = inner.read_u32()?;
                read += 1;
                if produced + run > size_y {
                    return Err(invalid(format!(
                        "voxel run of {run} overruns the {size_y}-cell column"
                    )));
                }
                voxels.extend(iter::repeat_n(voxel_from_le(color), run));
                produced += run;
            } else {
                if produced >= size_y {
                    return Err(invalid(format!(
                        "voxel column holds more than its {size_y} cells"
                    )));
                }
                voxels.push(QbclVoxel { r, g, b, mask });
                produced += 1;
            }
        }
        if produced != size_y {
            return Err(invalid(format!(
                "voxel column produced {produced} cells, but size {size:?} needs {size_y}"
            )));
        }
    }

    if !inner.is_empty() {
        return Err(invalid(format!(
            "{} unexpected trailing bytes in the voxel stream",
            inner.remaining().len()
        )));
    }
    debug_assert_eq!(voxels.len(), total);

    Ok(voxels)
}

/// Decodes a stream integer's four little-endian bytes (`r`, `g`, `b`, `mask`)
/// into a voxel.
fn voxel_from_le(value: u32) -> QbclVoxel {
    let [r, g, b, mask] = value.to_le_bytes();
    QbclVoxel { r, g, b, mask }
}

/// The number of pixels in a `width` by `height` image, or an error if it
/// overflows `usize`.
fn pixel_count(width: u32, height: u32) -> Result<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| {
            invalid(format!(
                "thumbnail {width}x{height} overflows the pixel range"
            ))
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

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl,
        qbcl::{from_qbcl_file_bytes, to_qbcl_file_bytes},
    };
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
    };

    fn solid(r: u8, g: u8, b: u8) -> QbclVoxel {
        QbclVoxel::new(r, g, b, 0x7e)
    }

    /// Exercises a model with a matrix child, a compound with a nested matrix
    /// child, a thumbnail, metadata, a non-default guid, runs and a voxel whose
    /// mask byte collides with the RLE marker.
    fn sample_file() -> QbclFile {
        // A solid voxel whose mask byte is 2 (the RLE marker); the encoder must
        // escape it as a one-cell run so it round-trips.
        let masky = QbclVoxel::new(9, 8, 7, 2);
        QbclFile {
            program_version: 0x03_01_02_00,
            file_version: 2,
            thumbnail: QbclThumbnail {
                width: 2,
                height: 1,
                pixels: vec![
                    QbclColor::new(10, 20, 30, 40),
                    QbclColor::new(255, 0, 0, 255),
                ],
            },
            metadata: QbclMetadata {
                title: "title".to_owned(),
                description: "desc".to_owned(),
                tags: "a,b".to_owned(),
                author: "me".to_owned(),
                company: String::new(),
                website: "example.com".to_owned(),
                copyright: "(c) 2026".to_owned(),
            },
            guid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            root: QbclNode {
                name: "root".to_owned(),
                visible: true,
                locked: false,
                body: QbclNodeBody::Model(QbclModel {
                    transform: QbclModel::DEFAULT_TRANSFORM,
                    children: vec![
                        QbclNode {
                            name: "mat".to_owned(),
                            visible: false,
                            locked: true,
                            // size [1, 4, 1]: one column of four cells along y,
                            // a run of two solids then an empty then masky.
                            body: QbclNodeBody::Matrix(QbclMatrix {
                                size: [1, 4, 1],
                                position: [1, -2, 3],
                                pivot: [0.5, 1.0, 1.5],
                                voxels: vec![
                                    solid(200, 10, 10),
                                    solid(200, 10, 10),
                                    QbclVoxel::default(),
                                    masky,
                                ],
                            }),
                        },
                        QbclNode {
                            name: "comp".to_owned(),
                            visible: true,
                            locked: false,
                            body: QbclNodeBody::Compound(QbclCompound {
                                matrix: QbclMatrix {
                                    size: [2, 1, 2],
                                    position: [0, 0, 0],
                                    pivot: [0.0, 0.0, 0.0],
                                    voxels: vec![
                                        solid(1, 2, 3),
                                        QbclVoxel::default(),
                                        QbclVoxel::default(),
                                        solid(4, 5, 6),
                                    ],
                                },
                                children: vec![QbclNode {
                                    name: "child".to_owned(),
                                    visible: true,
                                    locked: false,
                                    body: QbclNodeBody::Matrix(QbclMatrix {
                                        size: [1, 1, 1],
                                        position: [5, 5, 5],
                                        pivot: [0.25, 0.5, 0.75],
                                        voxels: vec![solid(9, 9, 9)],
                                    }),
                                }],
                            }),
                        },
                    ],
                }),
            },
        }
    }

    #[test]
    fn round_trips_full_file() {
        let file = sample_file();
        let bytes = to_qbcl_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_qbcl_file_bytes(&DependenciesImpl, &bytes).unwrap(),
            file
        );
    }

    #[test]
    fn round_trips_empty_file() {
        let file = QbclFile::default();
        let bytes = to_qbcl_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_qbcl_file_bytes(&DependenciesImpl, &bytes).unwrap(),
            file
        );
    }

    #[test]
    fn indexes_voxels_by_coordinate() {
        let decoded = from_qbcl_file_bytes(
            &DependenciesImpl,
            &to_qbcl_file_bytes(&DependenciesImpl, &sample_file()),
        )
        .unwrap();
        let QbclNodeBody::Model(model) = &decoded.root.body else {
            panic!("root is a model");
        };
        let QbclNodeBody::Matrix(matrix) = &model.children[0].body else {
            panic!("first child is a matrix");
        };
        assert_eq!(matrix.voxel(0, 0, 0), Some(solid(200, 10, 10)));
        assert_eq!(matrix.voxel(0, 1, 0), Some(solid(200, 10, 10)));
        assert_eq!(matrix.voxel(0, 2, 0), Some(QbclVoxel::default()));
        assert_eq!(matrix.voxel(0, 3, 0), Some(QbclVoxel::new(9, 8, 7, 2)));
        assert_eq!(matrix.voxel(0, 4, 0), None);
    }

    #[test]
    fn preserves_node_flags() {
        let decoded = from_qbcl_file_bytes(
            &DependenciesImpl,
            &to_qbcl_file_bytes(&DependenciesImpl, &sample_file()),
        )
        .unwrap();
        let QbclNodeBody::Model(model) = &decoded.root.body else {
            panic!("root is a model");
        };
        assert!(!model.children[0].visible);
        assert!(model.children[0].locked);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = to_qbcl_file_bytes(&DependenciesImpl, &QbclFile::default());
        bytes[0] = b'X';
        assert!(from_qbcl_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut bytes = to_qbcl_file_bytes(&DependenciesImpl, &QbclFile::default());
        // The file version is the u32 after the 4-byte magic and program version.
        bytes[8] = 9;
        assert!(from_qbcl_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = to_qbcl_file_bytes(&DependenciesImpl, &QbclFile::default());
        bytes.push(0);
        assert!(from_qbcl_file_bytes(&DependenciesImpl, &bytes).is_err());
    }

    #[test]
    fn rejects_truncated() {
        assert!(from_qbcl_file_bytes(&DependenciesImpl, b"").is_err());
        assert!(from_qbcl_file_bytes(&DependenciesImpl, b"QBCL").is_err());
    }
}
