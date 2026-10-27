use crate::{
    ByteWriter, CompressZlib, NODE_COMPOUND, NODE_MATRIX, NODE_MODEL, Result, check_voxel_count,
};
use qbcl::qbt::{QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode};

/// Serializes a [`QbtFile`] to a Qubicle Binary Tree `.qbt` file through
/// `dependencies`, the inverse of
/// [`from_qbt_file_bytes`](crate::qbt::from_qbt_file_bytes()).
///
/// Writes the header, `COLORMAP`, and `DATATREE` in turn, zlib-compressing each
/// matrix's voxel grid. The compressed bytes may differ from another encoder's
/// (compression is an encoder choice) but decode to the same grid, so a decoded
/// file re-encodes to an equivalent one. A matrix whose grid does not match its
/// size, or a length too large for its field, is an error.
pub fn to_qbt_file_bytes<D: CompressZlib>(dependencies: &D, file: &QbtFile) -> Result<Vec<u8>> {
    let mut out = ByteWriter::new();

    out.write_bytes(b"QB 2");
    out.write_u8(file.version.0);
    out.write_u8(file.version.1);
    for value in file.global_scale {
        out.write_f32(value);
    }

    out.write_bytes(b"COLORMAP");
    out.write_len(file.color_map.len())?;
    for color in &file.color_map {
        out.write_bytes(&[color.r, color.g, color.b, color.a]);
    }

    out.write_bytes(b"DATATREE");
    write_node(dependencies, &mut out, &file.root)?;

    Ok(out.into_bytes())
}

/// Writes one node: its type id, the byte length of its body, then the body.
/// The body is built into its own buffer first so its length is known; that
/// length spans the node's whole subtree.
fn write_node<D: CompressZlib>(
    dependencies: &D,
    out: &mut ByteWriter,
    node: &QbtNode,
) -> Result<()> {
    let (type_id, body) = match node {
        QbtNode::Matrix(matrix) => (
            NODE_MATRIX,
            build(|body| write_matrix(dependencies, body, matrix))?,
        ),

        QbtNode::Model(model) => (
            NODE_MODEL,
            build(|body| write_model(dependencies, body, model))?,
        ),

        QbtNode::Compound(compound) => (
            NODE_COMPOUND,
            build(|body| write_compound(dependencies, body, compound))?,
        ),

        QbtNode::Unknown(unknown) => (unknown.type_id, unknown.data.clone()),
    };
    out.write_u32(type_id);
    out.write_len(body.len())?;
    out.write_bytes(&body);

    Ok(())
}

/// Builds a byte buffer by running `fill` against a fresh writer.
fn build(fill: impl FnOnce(&mut ByteWriter) -> Result<()>) -> Result<Vec<u8>> {
    let mut writer = ByteWriter::new();
    fill(&mut writer)?;

    Ok(writer.into_bytes())
}

/// Writes a matrix node body: name, transform, size, then its compressed grid.
fn write_matrix<D: CompressZlib>(
    dependencies: &D,
    out: &mut ByteWriter,
    matrix: &QbtMatrix,
) -> Result<()> {
    check_voxel_count(matrix.size, matrix.voxels.len())?;
    let name = matrix.name.as_bytes();
    out.write_len(name.len())?;
    out.write_bytes(name);
    for value in matrix.position {
        out.write_i32(value);
    }
    for value in matrix.local_scale {
        out.write_u32(value);
    }
    for value in matrix.pivot {
        out.write_f32(value);
    }
    for value in matrix.size {
        out.write_u32(value);
    }

    let mut raw = Vec::with_capacity(matrix.voxels.len() * 4);
    for &voxel in &matrix.voxels {
        raw.extend_from_slice(&[voxel.r, voxel.g, voxel.b, voxel.mask]);
    }
    let compressed = dependencies.compress_zlib(&raw);
    out.write_len(compressed.len())?;
    out.write_bytes(&compressed);

    Ok(())
}

/// Writes a model node body: a child count then each child node.
fn write_model<D: CompressZlib>(
    dependencies: &D,
    out: &mut ByteWriter,
    model: &QbtModel,
) -> Result<()> {
    out.write_len(model.children.len())?;
    for child in &model.children {
        write_node(dependencies, out, child)?;
    }

    Ok(())
}

/// Writes a compound node body: a matrix grid then a child count and child nodes.
fn write_compound<D: CompressZlib>(
    dependencies: &D,
    out: &mut ByteWriter,
    compound: &QbtCompound,
) -> Result<()> {
    write_matrix(dependencies, out, &compound.matrix)?;
    out.write_len(compound.children.len())?;
    for child in &compound.children {
        write_node(dependencies, out, child)?;
    }

    Ok(())
}
