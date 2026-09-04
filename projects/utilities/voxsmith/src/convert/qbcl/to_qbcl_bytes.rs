use crate::{QBCL_DEPENDENCIES, QbclVoxMain, Result};
use qbcl_voxcore::codec::to_qbcl_bytes as raw_to_qbcl_bytes;

/// Writes a [`QbclVoxMain`] to the bytes of a Qubicle Construction
/// Library `.qbcl` file, the inverse of
/// [`from_qbcl_bytes`](crate::from_qbcl_bytes). The file is the one
/// [`to_qbcl_file`](crate::to_qbcl_file) builds.
pub fn to_qbcl_bytes(state: &QbclVoxMain) -> Result<Vec<u8>> {
    Ok(raw_to_qbcl_bytes(&QBCL_DEPENDENCIES, state)?)
}
