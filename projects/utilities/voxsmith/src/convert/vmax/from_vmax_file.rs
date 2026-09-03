use crate::{Result, VMaxFile, VoxelMaxVoxMain};
use vmax_voxcore::from_vmax_file as raw_from_vmax_file;

/// Loads a [`VMaxFile`] into a [`VoxelMaxVoxMain`], stashing the Voxel Max
/// state with no native voxcore home in the ext.
pub fn from_vmax_file(file: &VMaxFile) -> Result<VoxelMaxVoxMain> {
    Ok(raw_from_vmax_file(file)?)
}
