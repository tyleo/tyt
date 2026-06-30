use crate::{Result, to_qbt_file};
use qbcl_codec::qbt::to_qbt_file_bytes;
use voxcore::VoxMain;

/// Writes a [`VoxMain`] to the bytes of a Qubicle Binary Tree `.qbt` file, the
/// inverse of [`from_qbt_bytes`](crate::from_qbt_bytes). The state is written
/// back to a [`QbtFile`](qbcl::qbt::QbtFile) and encoded with [`qbcl_codec`].
pub fn to_qbt_bytes(state: &VoxMain) -> Result<Vec<u8>> {
    let file = to_qbt_file(state)?;
    Ok(to_qbt_file_bytes(&file))
}
