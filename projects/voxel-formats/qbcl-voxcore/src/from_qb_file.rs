use crate::{Result, read_qb};
use qbcl::qb::QbFile;
use voxcore::VoxMain;

/// Loads a decoded Qubicle Binary [`QbFile`] into a bare [`VoxMain`]. Each
/// matrix becomes an object sharing one `baseColor` palette, placed by a
/// hierarchy node at the matrix's scene position. Dropping the rest of the
/// Qubicle state leaves [`to_qb_file`](crate::to_qb_file) nothing to rebuild
/// a file from. The `ext` feature's `ext::from_qb_file_with_ext` keeps that
/// state instead.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn from_qb_file(file: &QbFile) -> Result<VoxMain<()>> {
    let (state, _) = read_qb(file)?;

    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::from_qb_file;
    use qbcl::qb::{QbFile, QbMatrix};

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbFile {
            matrices: vec![QbMatrix {
                name: String::new(),
                size: [2048, 2048, 2048],
                position: [0, 0, 0],
                voxels: Vec::new(),
            }],
            ..Default::default()
        };
        assert!(from_qb_file(&file).is_err());
    }
}
