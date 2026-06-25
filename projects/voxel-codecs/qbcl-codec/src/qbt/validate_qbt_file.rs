use crate::{Result, invalid};
use qbcl::qbt::{QbtFile, QbtMatrix, QbtNode};

/// Checks a decoded [`QbtFile`] for the size / voxel-count mismatch the byte
/// layout cannot catch: every matrix and compound grid must hold exactly
/// `size[0] * size[1] * size[2]` cells.
///
/// Decoding always produces grids of the right length, but a hand-built or
/// edited [`QbtFile`] can hold a grid whose length disagrees with its size,
/// which [`to_qbt_file_bytes`](crate::qbt::to_qbt_file_bytes) would write as a
/// structurally valid but broken file. Decoding does not call this; run it when
/// you need the guarantee.
pub fn validate_qbt_file(file: &QbtFile) -> Result<()> {
    validate_node(&file.root)
}

/// Recursively checks a node and its descendants.
fn validate_node(node: &QbtNode) -> Result<()> {
    match node {
        QbtNode::Matrix(matrix) => validate_matrix(matrix),
        QbtNode::Model(model) => model.children.iter().try_for_each(validate_node),
        QbtNode::Compound(compound) => {
            validate_matrix(&compound.matrix)?;
            compound.children.iter().try_for_each(validate_node)
        }
        QbtNode::Unknown(_) => Ok(()),
    }
}

/// Checks one matrix grid's length against its size.
fn validate_matrix(matrix: &QbtMatrix) -> Result<()> {
    let expected = (matrix.size[0] as usize)
        .checked_mul(matrix.size[1] as usize)
        .and_then(|xy| xy.checked_mul(matrix.size[2] as usize))
        .ok_or_else(|| {
            invalid(format!(
                "matrix {:?} size {:?} overflows the addressable range",
                matrix.name, matrix.size
            ))
        })?;
    if matrix.voxels.len() != expected {
        return Err(invalid(format!(
            "matrix {:?} holds {} voxels but its size {:?} needs {expected}",
            matrix.name,
            matrix.voxels.len(),
            matrix.size
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::qbt::validate_qbt_file;
    use qbcl::qbt::{QbtFile, QbtMatrix, QbtModel, QbtNode, QbtVoxel};

    fn file_with_matrix(matrix: QbtMatrix) -> QbtFile {
        QbtFile {
            root: QbtNode::Model(QbtModel {
                children: vec![QbtNode::Matrix(matrix)],
            }),
            ..Default::default()
        }
    }

    #[test]
    fn accepts_a_matching_grid() {
        let file = file_with_matrix(QbtMatrix {
            size: [2, 1, 1],
            voxels: vec![QbtVoxel::new(1, 2, 3, 0x7e), QbtVoxel::default()],
            ..Default::default()
        });
        assert!(validate_qbt_file(&file).is_ok());
    }

    #[test]
    fn accepts_an_empty_file() {
        assert!(validate_qbt_file(&QbtFile::default()).is_ok());
    }

    #[test]
    fn rejects_a_size_voxel_count_mismatch() {
        let file = file_with_matrix(QbtMatrix {
            size: [2, 2, 2],
            voxels: vec![QbtVoxel::default()],
            ..Default::default()
        });
        assert!(validate_qbt_file(&file).is_err());
    }
}
