use crate::{Result, VoxDocumentFile, codec::into_ext_slot};
use qbcl_voxcore::codec::to_qb_bytes;
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Encodes a state as a `.qb` file.
pub fn write_qb<T: VoxExtSlot>(state: VoxMain<T>) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qb_bytes(&into_ext_slot(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
