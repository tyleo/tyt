use crate::{QbtFile, QubicleQbtVoxMain, Result};
use qbcl_voxcore::from_qbt_file as raw_from_qbt_file;

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a
/// [`QubicleQbtVoxMain`], stashing the `.qbt` state with no native voxcore
/// home in the ext.
pub fn from_qbt_file(file: &QbtFile) -> Result<QubicleQbtVoxMain> {
    Ok(raw_from_qbt_file(file)?)
}
