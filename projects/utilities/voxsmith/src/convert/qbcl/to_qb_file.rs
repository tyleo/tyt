use crate::{QbFile, QubicleQbVoxMain, Result};
use qbcl_voxcore::to_qb_file as raw_to_qb_file;

/// Writes a [`QubicleQbVoxMain`] back to a decoded Qubicle Binary [`QbFile`],
/// the inverse of [`from_qb_file`](crate::from_qb_file). Requires the ext the
/// loader stashes.
pub fn to_qb_file(state: &QubicleQbVoxMain) -> Result<QbFile> {
    Ok(raw_to_qb_file(state)?)
}
