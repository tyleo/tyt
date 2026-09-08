use crate::{Result, VoxDocumentFile, single_file_bytes};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbcl_bytes};
use voxcore::VoxMain;

/// Decodes a `.qbcl` file into a bare state.
pub fn read_qbcl<D: DecompressZlib>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<()>> {
    Ok(from_qbcl_bytes(dependencies, single_file_bytes(files)?)?)
}
