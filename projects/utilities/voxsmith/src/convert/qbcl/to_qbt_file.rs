use crate::{QbtFile, QbtVoxMain, Result};
use qbcl_voxcore::to_qbt_file as raw_to_qbt_file;

/// Writes a [`QbtVoxMain`] back to a decoded Qubicle Binary Tree
/// [`QbtFile`], the inverse of [`from_qbt_file`](crate::from_qbt_file).
/// Requires the ext the loader stashes.
pub fn to_qbt_file(state: &QbtVoxMain) -> Result<QbtFile> {
    Ok(raw_to_qbt_file(state)?)
}
