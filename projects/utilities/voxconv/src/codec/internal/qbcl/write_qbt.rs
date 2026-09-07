use crate::{Result, VoxDocumentFile, codec::retype_ext};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbt_bytes};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.qbt` file.
pub fn write_qbt<D: CompressZlib, T: VoxExtBlockCodec>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbt_bytes(dependencies, &retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
