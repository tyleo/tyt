use crate::{QubicleQbclVoxMain, Result, to_qbcl_file};
use qbcl_codec::qbcl::to_qbcl_file_bytes;

/// Writes a [`QubicleQbclVoxMain`] to the bytes of a Qubicle Construction
/// Library `.qbcl` file, the inverse of
/// [`from_qbcl_bytes`](crate::from_qbcl_bytes). The state is written back to a
/// [`QbclFile`](qbcl::qbcl::QbclFile) and encoded with [`qbcl_codec`].
pub fn to_qbcl_bytes(state: &QubicleQbclVoxMain) -> Result<Vec<u8>> {
    let file = to_qbcl_file(state)?;
    Ok(to_qbcl_file_bytes(&file))
}
