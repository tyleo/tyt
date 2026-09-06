use crate::{
    Result, VoxDocumentFile,
    codec::{into_ext_slot, single_file_bytes},
};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbcl_bytes};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.qbcl` file into a state.
pub fn read_qbcl<D: DecompressZlib, T: VoxExtSlot>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    into_ext_slot(from_qbcl_bytes(dependencies, single_file_bytes(files)?)?)
}
