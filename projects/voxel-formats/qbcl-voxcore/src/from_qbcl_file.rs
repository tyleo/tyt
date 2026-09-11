use crate::{Result, read_qbcl};
use qbcl::qbcl::QbclFile;
use voxcore::VoxMain;

/// Loads a decoded Qubicle Construction Library [`QbclFile`] into a bare
/// [`VoxMain`]. Matrix and compound grids become objects sharing one
/// `baseColor` palette, and the scene tree becomes the hierarchy nodes. The
/// rest of the Qubicle state is dropped, so
/// [`to_qbcl_file`](crate::to_qbcl_file) writes the state back as a synthesized
/// file. [`ext::from_qbcl_file_with_ext`](crate::ext::from_qbcl_file_with_ext)
/// keeps that state instead.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn from_qbcl_file(file: &QbclFile) -> Result<VoxMain<()>> {
    read_qbcl(file)
}

#[cfg(test)]
mod tests {
    use crate::from_qbcl_file;
    use qbcl::qbcl::{QbclFile, QbclMatrix, QbclNode, QbclNodeBody};

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbclFile {
            root: QbclNode {
                body: QbclNodeBody::Matrix(QbclMatrix {
                    size: [2048, 2048, 2048],
                    position: [0, 0, 0],
                    pivot: [0.0, 0.0, 0.0],
                    voxels: Vec::new(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(from_qbcl_file(&file).is_err());
    }
}
