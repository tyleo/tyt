use crate::{ByteWriter, CompressZlib};
use qbcl::qbcl::{
    QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};

/// The matrix node type id.
const NODE_MATRIX: u32 = 0;

/// The model node type id.
const NODE_MODEL: u32 = 1;

/// The compound node type id.
const NODE_COMPOUND: u32 = 2;

/// The `u32` after every node's type id; always `1` in reference files.
const NODE_RESERVED: u32 = 1;

/// The middle of a node's three flag bytes (visible, this, locked); always `1`.
const NODE_FLAG: u8 = 1;

/// The RLE marker placed in a stream integer's mask (high) byte.
const RLE_MASK: u32 = 2;

/// The longest run a single RLE marker can encode (its length is one byte).
const MAX_RUN: usize = 255;

/// Serializes a [`QbclFile`] to a Qubicle Construction Library `.qbcl` file
/// through `dependencies`, the inverse of
/// [`from_qbcl_file_bytes`](crate::qbcl::from_qbcl_file_bytes).
///
/// Writes the header, thumbnail, metadata, and scene tree in turn,
/// run-length-encoding and zlib-compressing each matrix's voxel grid. The
/// compressed bytes may differ from another encoder's (compression and run
/// boundaries are encoder choices) but decode to the same grid, so a decoded
/// file re-encodes to an equivalent one.
pub fn to_qbcl_file_bytes<D: CompressZlib>(dependencies: &D, file: &QbclFile) -> Vec<u8> {
    let mut out = ByteWriter::new();

    out.write_bytes(b"QBCL");
    out.write_u32(file.program_version);
    out.write_u32(file.file_version);
    write_thumbnail(&mut out, &file.thumbnail);
    write_metadata(&mut out, &file.metadata);
    out.write_bytes(&file.guid);
    write_node(dependencies, &mut out, &file.root);

    out.into_bytes()
}

/// Writes the thumbnail: its dimensions then its pixels in `BGRA` order.
fn write_thumbnail(out: &mut ByteWriter, thumbnail: &QbclThumbnail) {
    out.write_u32(thumbnail.width);
    out.write_u32(thumbnail.height);
    for pixel in &thumbnail.pixels {
        out.write_bytes(&[pixel.b, pixel.g, pixel.r, pixel.a]);
    }
}

/// Writes the seven metadata strings, in the order the editor's UI lists them.
fn write_metadata(out: &mut ByteWriter, metadata: &QbclMetadata) {
    for value in [
        &metadata.title,
        &metadata.description,
        &metadata.tags,
        &metadata.author,
        &metadata.company,
        &metadata.website,
        &metadata.copyright,
    ] {
        write_len_string(out, value);
    }
}

/// Writes a `u32`-length-prefixed string.
fn write_len_string(out: &mut ByteWriter, value: &str) {
    let bytes = value.as_bytes();
    out.write_len(bytes.len());
    out.write_bytes(bytes);
}

/// Writes one node: the common header (type, name, and editor flags) then a
/// type-specific body.
fn write_node<D: CompressZlib>(dependencies: &D, out: &mut ByteWriter, node: &QbclNode) {
    let type_id = match node.body {
        QbclNodeBody::Matrix(_) => NODE_MATRIX,
        QbclNodeBody::Model(_) => NODE_MODEL,
        QbclNodeBody::Compound(_) => NODE_COMPOUND,
    };
    out.write_u32(type_id);
    out.write_u32(NODE_RESERVED);
    write_len_string(out, &node.name);
    out.write_u8(node.visible as u8);
    out.write_u8(NODE_FLAG);
    out.write_u8(node.locked as u8);

    match &node.body {
        QbclNodeBody::Matrix(matrix) => write_matrix(dependencies, out, matrix),
        QbclNodeBody::Model(model) => write_model(dependencies, out, model),
        QbclNodeBody::Compound(compound) => write_compound(dependencies, out, compound),
    }
}

/// Writes a matrix body: size, position, pivot, then the compressed voxel grid.
fn write_matrix<D: CompressZlib>(dependencies: &D, out: &mut ByteWriter, matrix: &QbclMatrix) {
    for value in matrix.size {
        out.write_u32(value);
    }
    for value in matrix.position {
        out.write_i32(value);
    }
    for value in matrix.pivot {
        out.write_f32(value);
    }
    let compressed = dependencies.compress_zlib(&encode_voxels(matrix));
    out.write_len(compressed.len());
    out.write_bytes(&compressed);
}

/// Writes a model body: the 36-byte transform chunk then the child nodes.
fn write_model<D: CompressZlib>(dependencies: &D, out: &mut ByteWriter, model: &QbclModel) {
    out.write_bytes(&model.transform);
    write_children(dependencies, out, &model.children);
}

/// Writes a compound body: a matrix grid then the child nodes.
fn write_compound<D: CompressZlib>(
    dependencies: &D,
    out: &mut ByteWriter,
    compound: &QbclCompound,
) {
    write_matrix(dependencies, out, &compound.matrix);
    write_children(dependencies, out, &compound.children);
}

/// Writes a `u32` child count then each child node.
fn write_children<D: CompressZlib>(dependencies: &D, out: &mut ByteWriter, children: &[QbclNode]) {
    out.write_len(children.len());
    for child in children {
        write_node(dependencies, out, child);
    }
}

/// Run-length-encodes a matrix's voxel grid into the raw bytes that the zlib
/// stream wraps, one column at a time. A run of equal cells becomes a marker
/// integer (mask byte [`RLE_MASK`], length in the low byte) followed by the
/// cell; a lone cell is written directly unless its own mask byte collides with
/// [`RLE_MASK`], in which case it too is escaped as a one-cell run so it round
/// trips. Runs longer than [`MAX_RUN`] are split.
fn encode_voxels(matrix: &QbclMatrix) -> Vec<u8> {
    let [size_x, size_y, size_z] = matrix.size.map(|value| value as usize);
    let columns = size_x.saturating_mul(size_z);

    let mut out = ByteWriter::new();
    for column in 0..columns {
        let base = column.saturating_mul(size_y);
        let cells = matrix
            .voxels
            .get(base..base.saturating_add(size_y))
            .unwrap_or(&[]);

        let mut column_data = ByteWriter::new();
        let mut integers = 0usize;
        let mut i = 0;
        while i < cells.len() {
            let value = voxel_to_u32(cells[i]);
            let mut run = 1;
            while i + run < cells.len() && voxel_to_u32(cells[i + run]) == value {
                run += 1;
            }
            let mut left = run;
            while left > 0 {
                let chunk = left.min(MAX_RUN);
                if chunk > 1 || cells[i].mask == RLE_MASK as u8 {
                    column_data.write_u32(RLE_MASK << 24 | chunk as u32);
                    column_data.write_u32(value);
                    integers += 2;
                } else {
                    column_data.write_u32(value);
                    integers += 1;
                }
                left -= chunk;
            }
            i += run;
        }

        debug_assert!(
            integers <= u16::MAX as usize,
            "voxel column has {integers} integers; the .qbcl format stores the count in a u16"
        );
        out.write_u16(integers as u16);
        out.write_bytes(&column_data.into_bytes());
    }

    out.into_bytes()
}

/// Encodes a voxel into its four little-endian stream bytes (`r`, `g`, `b`,
/// `mask`).
fn voxel_to_u32(voxel: QbclVoxel) -> u32 {
    u32::from_le_bytes([voxel.r, voxel.g, voxel.b, voxel.mask])
}
