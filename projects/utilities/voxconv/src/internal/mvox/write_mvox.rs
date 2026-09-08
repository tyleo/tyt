use crate::{Result, VoxDocumentFile};
use mvox_voxcore::codec::to_mvox_bytes;
use voxcore::VoxMain;

/// Encodes a bare state as a `.vox` file. The codec needs no dependencies.
pub fn write_mvox<D>(_dependencies: &D, state: &VoxMain<()>) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_mvox_bytes(state)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
