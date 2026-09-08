use crate::{
    Result,
    ext::{QbtVoxMain, from_qbt_file_with_ext},
};
use qbcl_codec::{DecompressZlib, qbt::from_qbt_file_bytes};

/// Loads the bytes of a Qubicle Binary Tree `.qbt` file through
/// `dependencies` into a [`QbtVoxMain`] carrying its ext, the bytes form of
/// [`from_qbt_file_with_ext`].
pub fn from_qbt_bytes_with_ext<D: DecompressZlib>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<QbtVoxMain> {
    let file = from_qbt_file_bytes(dependencies, bytes)?;

    from_qbt_file_with_ext(&file)
}
