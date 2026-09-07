use crate::{Result, VoxDocumentFile, retype_ext};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbcl_bytes};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.qbcl` file.
pub fn write_qbcl<D: CompressZlib, T: VoxExtBlockCodec>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbcl_bytes(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
