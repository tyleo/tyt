use crate::{Result, VoxDocumentFile, retype_ext, single_file_bytes};
use qbcl_voxcore::codec::{dependencies::DecompressZlib, from_qbcl_bytes_with_ext};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a `.qbcl` file into a state.
pub fn read_qbcl<D: DecompressZlib, T: VoxExtBlockCodec>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
    retype_ext(from_qbcl_bytes_with_ext(
        dependencies,
        single_file_bytes(files)?,
    )?)
}
