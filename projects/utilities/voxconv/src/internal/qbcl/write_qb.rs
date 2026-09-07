use crate::{Result, VoxDocumentFile, retype_ext};
use qbcl_voxcore::codec::to_qb_bytes;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.qb` file.
pub fn write_qb<T: VoxExtBlockCodec>(state: VoxMain<T>) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qb_bytes(&retype_ext(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
