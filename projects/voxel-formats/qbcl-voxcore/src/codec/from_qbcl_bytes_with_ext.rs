use crate::{
    Result,
    ext::{QbclVoxMain, from_qbcl_file_with_ext},
};
use qbcl_codec::{DecompressZlib, qbcl::from_qbcl_file_bytes};

/// Loads the bytes of a Qubicle Construction Library `.qbcl` file through
/// `dependencies` into a [`QbclVoxMain`] carrying its ext, the bytes form of
/// [`from_qbcl_file_with_ext`].
pub fn from_qbcl_bytes_with_ext<D: DecompressZlib>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<QbclVoxMain> {
    let file = from_qbcl_file_bytes(dependencies, bytes)?;

    from_qbcl_file_with_ext(&file)
}
