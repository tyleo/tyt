use crate::{Result, VoxDocumentFile};
use qbcl_voxcore::codec::to_qb_bytes;
use voxcore::VoxMain;

/// Encodes a bare state as a `.qb` file.
pub fn write_qb(state: &VoxMain<()>) -> Result<Vec<VoxDocumentFile>> {
    let bytes = to_qb_bytes(state)?;

    Ok(vec![VoxDocumentFile::single(bytes)])
}
