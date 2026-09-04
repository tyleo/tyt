use crate::{QBCL_DEPENDENCIES, QbtVoxMain, Result};
use qbcl_voxcore::codec::to_qbt_bytes as raw_to_qbt_bytes;

/// Writes a [`QbtVoxMain`] to the bytes of a Qubicle Binary Tree
/// `.qbt` file, the inverse of [`from_qbt_bytes`](crate::from_qbt_bytes). The
/// file is the one [`to_qbt_file`](crate::to_qbt_file) builds.
pub fn to_qbt_bytes(state: &QbtVoxMain) -> Result<Vec<u8>> {
    Ok(raw_to_qbt_bytes(&QBCL_DEPENDENCIES, state)?)
}
