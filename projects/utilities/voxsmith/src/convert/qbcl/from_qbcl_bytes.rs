use crate::{QBCL_DEPENDENCIES, QbclVoxMain, Result};
use qbcl_voxcore::codec::from_qbcl_bytes as raw_from_qbcl_bytes;

/// Loads the bytes of a Qubicle Construction Library `.qbcl` file into a
/// [`QbclVoxMain`]. The state is the one
/// [`from_qbcl_file`](crate::from_qbcl_file) loads.
pub fn from_qbcl_bytes(bytes: &[u8]) -> Result<QbclVoxMain> {
    Ok(raw_from_qbcl_bytes(&QBCL_DEPENDENCIES, bytes)?)
}
