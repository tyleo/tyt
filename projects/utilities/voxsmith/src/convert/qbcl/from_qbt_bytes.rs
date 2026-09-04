use crate::{QBCL_DEPENDENCIES, QbtVoxMain, Result};
use qbcl_voxcore::codec::from_qbt_bytes as raw_from_qbt_bytes;

/// Loads the bytes of a Qubicle Binary Tree `.qbt` file into a
/// [`QbtVoxMain`]. The state is the one
/// [`from_qbt_file`](crate::from_qbt_file) loads.
pub fn from_qbt_bytes(bytes: &[u8]) -> Result<QbtVoxMain> {
    Ok(raw_from_qbt_bytes(&QBCL_DEPENDENCIES, bytes)?)
}
