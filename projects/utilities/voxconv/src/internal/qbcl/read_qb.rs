use crate::{Result, VoxDocumentFile, single_file_bytes};
use qbcl_voxcore::codec::from_qb_bytes;
use voxcore::VoxMain;

/// Decodes a `.qb` file into a bare state.
pub fn read_qb(files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
    Ok(from_qb_bytes(single_file_bytes(files)?)?)
}
