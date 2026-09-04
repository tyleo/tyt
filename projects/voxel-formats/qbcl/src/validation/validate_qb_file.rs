use crate::{
    qb::QbFile,
    validation::{Error, Result},
};

/// Checks a decoded [`QbFile`]: every matrix's grid must hold exactly
/// `size[0] * size[1] * size[2]` cells.
pub fn validate_qb_file(file: &QbFile) -> Result<()> {
    for (index, matrix) in file.matrices.iter().enumerate() {
        let expected = (matrix.size[0] as usize)
            .checked_mul(matrix.size[1] as usize)
            .and_then(|xy| xy.checked_mul(matrix.size[2] as usize))
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "matrix {index} size {:?} overflows the addressable range",
                    matrix.size
                ))
            })?;
        if matrix.voxels.len() != expected {
            return Err(Error::Invalid(format!(
                "matrix {index} holds {} voxels but its size {:?} needs {expected}",
                matrix.voxels.len(),
                matrix.size
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        qb::{QbFile, QbMatrix, QbVoxel},
        validation::validate_qb_file,
    };

    #[test]
    fn accepts_a_matching_grid() {
        let file = QbFile {
            matrices: vec![QbMatrix {
                name: "m".to_owned(),
                size: [2, 1, 1],
                position: [0, 0, 0],
                voxels: vec![QbVoxel::new(1, 2, 3), QbVoxel::default()],
            }],
            ..Default::default()
        };
        assert!(validate_qb_file(&file).is_ok());
    }

    #[test]
    fn accepts_an_empty_file() {
        assert!(validate_qb_file(&QbFile::default()).is_ok());
    }

    #[test]
    fn rejects_a_size_voxel_count_mismatch() {
        let file = QbFile {
            matrices: vec![QbMatrix {
                name: "m".to_owned(),
                size: [2, 2, 2],
                position: [0, 0, 0],
                voxels: vec![QbVoxel::default()],
            }],
            ..Default::default()
        };
        assert!(validate_qb_file(&file).is_err());
    }
}
