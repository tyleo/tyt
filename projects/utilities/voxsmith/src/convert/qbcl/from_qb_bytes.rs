use crate::{QbVoxMain, Result};
use qbcl_voxcore::codec::from_qb_bytes as raw_from_qb_bytes;

/// Loads the bytes of a Qubicle Binary `.qb` file into a [`QbVoxMain`].
/// The state is the one [`from_qb_file`](crate::from_qb_file) loads.
pub fn from_qb_bytes(bytes: &[u8]) -> Result<QbVoxMain> {
    Ok(raw_from_qb_bytes(bytes)?)
}
