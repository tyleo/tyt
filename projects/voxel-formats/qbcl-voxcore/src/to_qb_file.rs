use crate::{Result, write_qb};
use qbcl::qb::QbFile;
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to a decoded Qubicle Binary [`QbFile`], the
/// inverse of [`from_qb_file`](crate::from_qb_file). This errors because a
/// Qubicle Binary file rebuilds only from its `qb` ext and a bare state
/// carries none. The `ext` feature's `ext::to_qb_file_with_ext` writes a
/// loaded file back exactly.
pub fn to_qb_file(state: &VoxMain<()>) -> Result<QbFile> {
    write_qb(state, None)
}

#[cfg(test)]
mod tests {
    use crate::to_qb_file;
    use voxcore::VoxMain;

    /// A bare state carries no `qb` ext, so the writer has nothing to rebuild
    /// the file from.
    #[test]
    fn errors_without_an_ext() {
        assert!(to_qb_file(&VoxMain::default()).is_err());
    }
}
