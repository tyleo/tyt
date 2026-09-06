use crate::{
    Result, VoxDocumentFile,
    codec::{into_ext_slot, single_file_bytes},
};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbt_bytes};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.qbt` file into a state.
pub fn read_qbt<D: DecompressZlib, T: VoxExtSlot>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    into_ext_slot(from_qbt_bytes(dependencies, single_file_bytes(files)?)?)
}
