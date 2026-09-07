use crate::{Result, VoxDocumentFile, retype_ext};
use mvox_voxcore::codec::to_mvox_bytes;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.vox` file. The codec needs no dependencies.
pub fn write_mvox<D, T: VoxExtBlockCodec>(
    _dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_mvox_bytes(&retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
