use crate::{
    qbcl::{QbclFile, QbclMatrix, QbclNode, QbclNodeBody, QbclThumbnail},
    validation::{Error, Result},
};

/// Checks a decoded [`QbclFile`]: the thumbnail must hold exactly
/// `width * height` pixels, and every matrix and compound grid must hold
/// exactly `size[0] * size[1] * size[2]` cells.
pub fn validate_qbcl_file(file: &QbclFile) -> Result<()> {
    validate_thumbnail(&file.thumbnail)?;
    validate_node(&file.root)
}

/// Checks the thumbnail's pixel count against its dimensions.
fn validate_thumbnail(thumbnail: &QbclThumbnail) -> Result<()> {
    let expected = (thumbnail.width as usize)
        .checked_mul(thumbnail.height as usize)
        .ok_or_else(|| {
            Error::Invalid(format!(
                "thumbnail {}x{} overflows the pixel range",
                thumbnail.width, thumbnail.height
            ))
        })?;
    if thumbnail.pixels.len() != expected {
        return Err(Error::Invalid(format!(
            "thumbnail holds {} pixels but its {}x{} size needs {expected}",
            thumbnail.pixels.len(),
            thumbnail.width,
            thumbnail.height
        )));
    }
    Ok(())
}

/// Recursively checks a node and its descendants.
fn validate_node(node: &QbclNode) -> Result<()> {
    match &node.body {
        QbclNodeBody::Matrix(matrix) => validate_matrix(matrix),
        QbclNodeBody::Model(model) => model.children.iter().try_for_each(validate_node),
        QbclNodeBody::Compound(compound) => {
            validate_matrix(&compound.matrix)?;
            compound.children.iter().try_for_each(validate_node)
        }
    }
}

/// Checks one matrix grid's length against its size.
fn validate_matrix(matrix: &QbclMatrix) -> Result<()> {
    let expected = (matrix.size[0] as usize)
        .checked_mul(matrix.size[1] as usize)
        .and_then(|xy| xy.checked_mul(matrix.size[2] as usize))
        .ok_or_else(|| {
            Error::Invalid(format!(
                "matrix size {:?} overflows the addressable range",
                matrix.size
            ))
        })?;
    if matrix.voxels.len() != expected {
        return Err(Error::Invalid(format!(
            "matrix holds {} voxels but its size {:?} needs {expected}",
            matrix.voxels.len(),
            matrix.size
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        qbcl::{
            QbclColor, QbclFile, QbclMatrix, QbclModel, QbclNode, QbclNodeBody, QbclThumbnail,
            QbclVoxel,
        },
        validation::validate_qbcl_file,
    };

    fn file_with_matrix(matrix: QbclMatrix) -> QbclFile {
        QbclFile {
            root: QbclNode {
                body: QbclNodeBody::Model(QbclModel {
                    children: vec![QbclNode {
                        body: QbclNodeBody::Matrix(matrix),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn accepts_a_matching_file() {
        let file = file_with_matrix(QbclMatrix {
            size: [2, 1, 1],
            voxels: vec![QbclVoxel::new(1, 2, 3, 0x7e), QbclVoxel::default()],
            ..Default::default()
        });
        assert!(validate_qbcl_file(&file).is_ok());
    }

    #[test]
    fn accepts_an_empty_file() {
        assert!(validate_qbcl_file(&QbclFile::default()).is_ok());
    }

    #[test]
    fn rejects_a_size_voxel_count_mismatch() {
        let file = file_with_matrix(QbclMatrix {
            size: [2, 2, 2],
            voxels: vec![QbclVoxel::default()],
            ..Default::default()
        });
        assert!(validate_qbcl_file(&file).is_err());
    }

    #[test]
    fn rejects_a_thumbnail_pixel_count_mismatch() {
        let file = QbclFile {
            thumbnail: QbclThumbnail {
                width: 2,
                height: 2,
                pixels: vec![QbclColor::default()],
            },
            ..Default::default()
        };
        assert!(validate_qbcl_file(&file).is_err());
    }
}
