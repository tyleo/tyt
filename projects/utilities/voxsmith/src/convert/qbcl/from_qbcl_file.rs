use crate::{QbclFile, QubicleQbclVoxMain, Result};
use qbcl_voxcore::from_qbcl_file as raw_from_qbcl_file;

/// Loads a decoded Qubicle Construction Library [`QbclFile`] into a
/// [`QubicleQbclVoxMain`], stashing the `.qbcl` state with no native voxcore
/// home in the ext.
pub fn from_qbcl_file(file: &QbclFile) -> Result<QubicleQbclVoxMain> {
    Ok(raw_from_qbcl_file(file)?)
}
