use crate::{Result, VoxDocumentFile, single_file_bytes};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbt_bytes};
use voxcore::VoxMain;

/// Decodes a `.qbt` file into a bare state.
pub fn read_qbt<D: DecompressZlib>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<()>> {
    Ok(from_qbt_bytes(dependencies, single_file_bytes(files)?)?)
}
