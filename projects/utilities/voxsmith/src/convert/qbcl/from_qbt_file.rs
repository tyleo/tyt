use crate::{QbtFile, QbtVoxMain, Result};
use qbcl_voxcore::from_qbt_file as raw_from_qbt_file;

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a
/// [`QbtVoxMain`], stashing the `.qbt` state with no native voxcore
/// home in the ext.
pub fn from_qbt_file(file: &QbtFile) -> Result<QbtVoxMain> {
    Ok(raw_from_qbt_file(file)?)
}
