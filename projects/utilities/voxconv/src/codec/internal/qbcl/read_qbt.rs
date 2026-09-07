use crate::{
    Result, VoxDocumentFile,
    codec::{retype_ext, single_file_bytes},
};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbt_bytes};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a `.qbt` file into a state.
pub fn read_qbt<D: DecompressZlib, T: VoxExtBlockCodec>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    retype_ext(from_qbt_bytes(dependencies, single_file_bytes(files)?)?)
}
