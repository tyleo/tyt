use crate::{Result, VoxDocumentFile, codec::into_ext_slot};
use goxl_voxcore::codec::{dependencies::EncodePng, to_goxl_bytes};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Encodes a state as a `.gox` file.
pub fn write_goxl<D: EncodePng, T: VoxExtSlot>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_goxl_bytes(dependencies, &into_ext_slot(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
