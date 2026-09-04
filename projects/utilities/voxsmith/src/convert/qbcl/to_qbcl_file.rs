use crate::{QbclFile, QbclVoxMain, Result};
use qbcl_voxcore::to_qbcl_file as raw_to_qbcl_file;

/// Writes a [`QbclVoxMain`] to a decoded Qubicle Construction Library
/// [`QbclFile`], the inverse of [`from_qbcl_file`](crate::from_qbcl_file). A
/// state without the ext, such as one loaded from another format, has its
/// file synthesized from the bare scene.
pub fn to_qbcl_file(state: &QbclVoxMain) -> Result<QbclFile> {
    Ok(raw_to_qbcl_file(state)?)
}
