use crate::{Result, write_qbt};
use qbcl::qbt::QbtFile;
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to a decoded Qubicle Binary Tree [`QbtFile`],
/// the inverse of [`from_qbt_file`](crate::from_qbt_file). This errors
/// because a Qubicle Binary Tree file rebuilds only from its `qbt` ext and a
/// bare state carries none. The `ext` feature's `ext::to_qbt_file_with_ext`
/// writes a loaded file back exactly.
pub fn to_qbt_file(state: &VoxMain<()>) -> Result<QbtFile> {
    write_qbt(state, None)
}

#[cfg(test)]
mod tests {
    use crate::to_qbt_file;
    use voxcore::VoxMain;

    /// A bare state carries no `qbt` ext, so the writer has nothing to
    /// rebuild the file from.
    #[test]
    fn errors_without_an_ext() {
        assert!(to_qbt_file(&VoxMain::default()).is_err());
    }
}
