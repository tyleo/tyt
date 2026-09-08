use crate::{Result, from_qbt_file};
use qbcl_codec::{DecompressZlib, qbt::from_qbt_file_bytes};
use voxcore::VoxMain;

/// Loads the bytes of a Qubicle Binary Tree `.qbt` file into a bare
/// [`VoxMain`] through `dependencies`, the bytes form of [`from_qbt_file`].
pub fn from_qbt_bytes<D: DecompressZlib>(dependencies: &D, bytes: &[u8]) -> Result<VoxMain<()>> {
    let file = from_qbt_file_bytes(dependencies, bytes)?;

    from_qbt_file(&file)
}
