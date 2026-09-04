use crate::{QbtVoxMain, Result, from_qbt_file};
use qbcl_codec::{DecompressZlib, qbt::from_qbt_file_bytes};

/// Loads the bytes of a Qubicle Binary Tree `.qbt` file into a
/// [`QbtVoxMain`] through `dependencies`, the bytes form of
/// [`from_qbt_file`].
pub fn from_qbt_bytes<D: DecompressZlib>(dependencies: &D, bytes: &[u8]) -> Result<QbtVoxMain> {
    let file = from_qbt_file_bytes(dependencies, bytes)?;
    from_qbt_file(&file)
}
