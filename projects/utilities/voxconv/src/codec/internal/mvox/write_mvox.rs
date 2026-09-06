use crate::{Result, VoxDocumentFile, codec::into_ext_slot};
use mvox_voxcore::codec::to_mvox_bytes;
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Encodes a state as a `.vox` file. The codec needs no dependencies.
pub fn write_mvox<D, T: VoxExtSlot>(
    _dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_mvox_bytes(&into_ext_slot(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
