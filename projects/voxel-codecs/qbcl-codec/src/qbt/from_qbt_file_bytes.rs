use crate::{ByteReader, Result, invalid, zlib_decompress};
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};

/// The matrix node type id.
const NODE_MATRIX: u32 = 0;

/// The model node type id.
const NODE_MODEL: u32 = 1;

/// The compound node type id.
const NODE_COMPOUND: u32 = 2;

/// The deepest the reader will descend, so a pathologically nested file is
/// rejected rather than overflowing the stack.
const MAX_DEPTH: usize = 4096;

/// Parses a Qubicle Binary Tree `.qbt` file into a [`QbtFile`].
///
/// Reads the header, the `COLORMAP` palette, and the `DATATREE` scene tree in
/// turn, zlib-decompressing each matrix's voxel grid. A node whose type id this
/// crate does not model is preserved verbatim on a [`QbtNode::Unknown`].
/// Parsing is bounds-checked, so a truncated or malformed file is rejected
/// rather than masked.
pub fn from_qbt_file_bytes(bytes: &[u8]) -> Result<QbtFile> {
    let mut reader = ByteReader::new(bytes);

    let magic = reader.read_array::<4>()?;
    if magic != *b"QB 2" {
        return Err(invalid(format!(
            "not a .qbt file: expected magic \"QB 2\", found {magic:?}"
        )));
    }
    let version = (reader.read_u8()?, reader.read_u8()?);
    let global_scale = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];

    let color_map = read_color_map(&mut reader)?;

    let caption = reader.read_array::<8>()?;
    if &caption != b"DATATREE" {
        return Err(invalid(format!(
            "expected a DATATREE section, found {caption:?}"
        )));
    }
    let root = read_node(&mut reader, 0)?;

    if !reader.is_empty() {
        return Err(invalid(format!(
            "{} unexpected trailing bytes after the scene tree",
            reader.remaining().len()
        )));
    }

    Ok(QbtFile {
        version,
        global_scale,
        color_map,
        root,
    })
}

/// Reads the `COLORMAP` section's caption and colors.
fn read_color_map(reader: &mut ByteReader) -> Result<Vec<QbtColor>> {
    let caption = reader.read_array::<8>()?;
    if &caption != b"COLORMAP" {
        return Err(invalid(format!(
            "expected a COLORMAP section, found {caption:?}"
        )));
    }
    let count = reader.read_u32()? as usize;
    let mut colors = Vec::with_capacity(count.min(reader.remaining().len() / 4));
    for _ in 0..count {
        let [r, g, b, a] = reader.read_array::<4>()?;
        colors.push(QbtColor::new(r, g, b, a));
    }
    Ok(colors)
}

/// Reads one node: its type id and data-size header, then the body. A known
/// node is parsed in place; an unknown node's `data_size` bytes are kept
/// verbatim.
fn read_node(reader: &mut ByteReader, depth: usize) -> Result<QbtNode> {
    if depth > MAX_DEPTH {
        return Err(invalid(format!(
            "scene tree is nested deeper than {MAX_DEPTH} nodes"
        )));
    }
    let type_id = reader.read_u32()?;
    let data_size = reader.read_u32()? as usize;
    match type_id {
        NODE_MATRIX => Ok(QbtNode::Matrix(read_matrix(reader)?)),
        NODE_MODEL => Ok(QbtNode::Model(read_model(reader, depth)?)),
        NODE_COMPOUND => Ok(QbtNode::Compound(read_compound(reader, depth)?)),
        other => Ok(QbtNode::Unknown(QbtUnknownNode {
            type_id: other,
            data: reader.read_bytes(data_size)?.to_vec(),
        })),
    }
}

/// Reads a matrix node body: name, transform, size, then its voxel grid.
fn read_matrix(reader: &mut ByteReader) -> Result<QbtMatrix> {
    let name_len = reader.read_u32()? as usize;
    let name = reader.read_string(name_len)?;
    let position = [reader.read_i32()?, reader.read_i32()?, reader.read_i32()?];
    let local_scale = [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?];
    let pivot = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];
    let size = [reader.read_u32()?, reader.read_u32()?, reader.read_u32()?];
    let voxel_data_size = reader.read_u32()? as usize;
    let voxels = read_voxels(size, reader.read_bytes(voxel_data_size)?)?;
    Ok(QbtMatrix {
        name,
        position,
        local_scale,
        pivot,
        size,
        voxels,
    })
}

/// Decompresses and splits a matrix's voxel data into its dense grid. The
/// decompressed length must match the size exactly.
fn read_voxels(size: [u32; 3], compressed: &[u8]) -> Result<Vec<QbtVoxel>> {
    let expected = voxel_count(size)?.checked_mul(4).ok_or_else(|| {
        invalid(format!(
            "matrix size {size:?} overflows the addressable range"
        ))
    })?;
    let raw = zlib_decompress(compressed)?;
    if raw.len() != expected {
        return Err(invalid(format!(
            "decompressed voxel data is {} bytes, but size {size:?} needs {expected}",
            raw.len()
        )));
    }
    Ok(raw
        .chunks_exact(4)
        .map(|cell| QbtVoxel {
            r: cell[0],
            g: cell[1],
            b: cell[2],
            mask: cell[3],
        })
        .collect())
}

/// Reads a model node body: a child count then that many child nodes.
fn read_model(reader: &mut ByteReader, depth: usize) -> Result<QbtModel> {
    Ok(QbtModel {
        children: read_children(reader, depth)?,
    })
}

/// Reads a compound node body: a matrix grid then a child count and child nodes.
fn read_compound(reader: &mut ByteReader, depth: usize) -> Result<QbtCompound> {
    let matrix = read_matrix(reader)?;
    let children = read_children(reader, depth)?;
    Ok(QbtCompound { matrix, children })
}

/// Reads a `u32` child count then that many child nodes, each one level deeper.
fn read_children(reader: &mut ByteReader, depth: usize) -> Result<Vec<QbtNode>> {
    let count = reader.read_u32()? as usize;
    // A child node is at least eight bytes (its type id and data size), so a
    // count larger than the remaining bytes allow is malformed.
    let mut children = Vec::with_capacity(count.min(reader.remaining().len() / 8));
    for _ in 0..count {
        children.push(read_node(reader, depth + 1)?);
    }
    Ok(children)
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

#[cfg(test)]
mod tests {
    use crate::qbt::{from_qbt_file_bytes, to_qbt_file_bytes};
    use qbcl::qbt::{
        QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
    };

    fn solid(r: u8, g: u8, b: u8) -> QbtVoxel {
        QbtVoxel::new(r, g, b, 0x7e)
    }

    /// Exercises a model with a matrix, a compound with a nested matrix child,
    /// an unknown node, a color map, and a non-trivial global scale.
    fn sample_file() -> QbtFile {
        QbtFile {
            version: (1, 0),
            global_scale: [1.0, 2.0, 0.5],
            color_map: vec![QbtColor::new(10, 20, 30, 40), QbtColor::new(255, 0, 0, 255)],
            root: QbtNode::Model(QbtModel {
                children: vec![
                    QbtNode::Matrix(QbtMatrix {
                        name: "mat".to_owned(),
                        position: [1, -2, 3],
                        local_scale: [1, 1, 1],
                        pivot: [0.5, 1.0, 1.5],
                        size: [2, 1, 1],
                        voxels: vec![solid(200, 10, 10), QbtVoxel::default()],
                    }),
                    QbtNode::Compound(QbtCompound {
                        matrix: QbtMatrix {
                            name: "comp".to_owned(),
                            position: [0, 0, 0],
                            local_scale: [1, 1, 1],
                            pivot: [0.0, 0.0, 0.0],
                            size: [1, 1, 1],
                            voxels: vec![solid(1, 2, 3)],
                        },
                        children: vec![QbtNode::Matrix(QbtMatrix {
                            name: "child".to_owned(),
                            position: [5, 5, 5],
                            local_scale: [2, 2, 2],
                            pivot: [0.25, 0.5, 0.75],
                            size: [1, 1, 2],
                            voxels: vec![QbtVoxel::default(), solid(9, 9, 9)],
                        })],
                    }),
                    QbtNode::Unknown(QbtUnknownNode {
                        type_id: 7,
                        data: vec![1, 2, 3, 4],
                    }),
                ],
            }),
        }
    }

    #[test]
    fn round_trips_full_file() {
        let file = sample_file();
        let bytes = to_qbt_file_bytes(&file);
        assert_eq!(from_qbt_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn round_trips_empty_file() {
        let file = QbtFile::default();
        let bytes = to_qbt_file_bytes(&file);
        assert_eq!(from_qbt_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn indexes_voxels_by_coordinate() {
        let file = sample_file();
        let decoded = from_qbt_file_bytes(&to_qbt_file_bytes(&file)).unwrap();
        let QbtNode::Model(model) = &decoded.root else {
            panic!("root is a model");
        };
        let QbtNode::Matrix(matrix) = &model.children[0] else {
            panic!("first child is a matrix");
        };
        assert_eq!(matrix.voxel(0, 0, 0), Some(solid(200, 10, 10)));
        assert_eq!(matrix.voxel(1, 0, 0), Some(QbtVoxel::default()));
        assert_eq!(matrix.voxel(2, 0, 0), None);
    }

    #[test]
    fn root_data_size_spans_whole_subtree() {
        // The reader ignores a known node's data-size field, so a round trip
        // cannot catch a wrong value; check it against the bytes directly. The
        // root node's size must cover every byte after its own header.
        let bytes = to_qbt_file_bytes(&sample_file());
        let mut pos = 4 + 2 + 12; // magic, version, global scale
        assert_eq!(&bytes[pos..pos + 8], b"COLORMAP");
        pos += 8;
        let color_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4 + color_count * 4;
        assert_eq!(&bytes[pos..pos + 8], b"DATATREE");
        pos += 8 + 4; // caption, then the root's type id
        let data_size = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        assert_eq!(data_size, bytes.len() - pos);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = to_qbt_file_bytes(&QbtFile::default());
        bytes[0] = b'X';
        assert!(from_qbt_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = to_qbt_file_bytes(&QbtFile::default());
        bytes.push(0);
        assert!(from_qbt_file_bytes(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated() {
        assert!(from_qbt_file_bytes(b"").is_err());
        assert!(from_qbt_file_bytes(b"QB 2").is_err());
    }
}
