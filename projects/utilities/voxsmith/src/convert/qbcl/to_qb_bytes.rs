use crate::{QubicleQbVoxMain, Result};
use qbcl_voxcore::codec::to_qb_bytes as raw_to_qb_bytes;

/// Writes a [`QubicleQbVoxMain`] to the bytes of a Qubicle Binary `.qb` file,
/// the inverse of [`from_qb_bytes`](crate::from_qb_bytes). The file is the one
/// [`to_qb_file`](crate::to_qb_file) builds.
pub fn to_qb_bytes(state: &QubicleQbVoxMain) -> Result<Vec<u8>> {
    Ok(raw_to_qb_bytes(state)?)
}
