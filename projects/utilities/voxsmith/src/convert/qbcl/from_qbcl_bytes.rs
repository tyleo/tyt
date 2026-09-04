use crate::{QBCL_DEPENDENCIES, QubicleQbclVoxMain, Result};
use qbcl_voxcore::codec::from_qbcl_bytes as raw_from_qbcl_bytes;

/// Loads the bytes of a Qubicle Construction Library `.qbcl` file into a
/// [`QubicleQbclVoxMain`]. The state is the one
/// [`from_qbcl_file`](crate::from_qbcl_file) loads.
pub fn from_qbcl_bytes(bytes: &[u8]) -> Result<QubicleQbclVoxMain> {
    Ok(raw_from_qbcl_bytes(&QBCL_DEPENDENCIES, bytes)?)
}
