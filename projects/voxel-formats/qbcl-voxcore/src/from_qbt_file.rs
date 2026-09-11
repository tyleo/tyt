use crate::{Result, read_qbt};
use qbcl::qbt::QbtFile;
use voxcore::VoxMain;

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a bare [`VoxMain`].
/// Matrix and compound grids become objects sharing one `baseColor` palette,
/// and the scene tree becomes the hierarchy nodes. The rest of the Qubicle
/// state is dropped, so [`to_qbt_file`](crate::to_qbt_file) writes the state
/// back as a synthesized file.
/// [`ext::from_qbt_file_with_ext`](crate::ext::from_qbt_file_with_ext) keeps
/// that state instead.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn from_qbt_file(file: &QbtFile) -> Result<VoxMain<()>> {
    read_qbt(file)
}

#[cfg(test)]
mod tests {
    use crate::from_qbt_file;
    use qbcl::qbt::{QbtFile, QbtMatrix, QbtNode};

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbtFile {
            root: QbtNode::Matrix(QbtMatrix {
                size: [2048, 2048, 2048],
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(from_qbt_file(&file).is_err());
    }
}
