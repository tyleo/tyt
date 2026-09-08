use crate::{Result, VoxDocumentFile, single_file_bytes};
use mvox_voxcore::codec::from_mvox_bytes;
use voxcore::VoxMain;

/// Decodes a `.vox` file into a bare state. The codec needs no dependencies.
pub fn read_mvox<D>(_dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
    Ok(from_mvox_bytes(single_file_bytes(files)?)?)
}
