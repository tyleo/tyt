use crate::{Result, VMaxFile, VMaxVoxMain};
use vmax_voxcore::from_vmax_file as raw_from_vmax_file;

/// Loads a [`VMaxFile`] into a [`VMaxVoxMain`], stashing the Voxel Max
/// state with no native voxcore home in the ext.
pub fn from_vmax_file(file: &VMaxFile) -> Result<VMaxVoxMain> {
    Ok(raw_from_vmax_file(file)?)
}
