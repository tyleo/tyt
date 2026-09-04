use crate::{QbclVoxMain, Result, from_qbcl_file};
use qbcl_codec::{DecompressZlib, qbcl::from_qbcl_file_bytes};

/// Loads the bytes of a Qubicle Construction Library `.qbcl` file into a
/// [`QbclVoxMain`] through `dependencies`, the bytes form of
/// [`from_qbcl_file`].
pub fn from_qbcl_bytes<D: DecompressZlib>(dependencies: &D, bytes: &[u8]) -> Result<QbclVoxMain> {
    let file = from_qbcl_file_bytes(dependencies, bytes)?;
    from_qbcl_file(&file)
}
