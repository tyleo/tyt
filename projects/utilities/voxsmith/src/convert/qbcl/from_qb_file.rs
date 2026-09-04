use crate::{QbFile, QubicleQbVoxMain, Result};
use qbcl_voxcore::from_qb_file as raw_from_qb_file;

/// Loads a decoded Qubicle Binary [`QbFile`] into a [`QubicleQbVoxMain`],
/// stashing the `.qb` state with no native voxcore home in the ext.
pub fn from_qb_file(file: &QbFile) -> Result<QubicleQbVoxMain> {
    Ok(raw_from_qb_file(file)?)
}
