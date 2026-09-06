use crate::{Result, VoxDocumentFile, codec::into_ext_slot};
use qbcl_voxcore::codec::{dependencies::CompressZlib, to_qbcl_bytes};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Encodes a state as a `.qbcl` file.
pub fn write_qbcl<D: CompressZlib, T: VoxExtSlot>(
    dependencies: &D,
    state: VoxMain<T>,
) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qbcl_bytes(dependencies, &into_ext_slot(state)?)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
