use crate::{Result, VMaxFile, VoxelMaxColorFormat, VoxelMaxVoxMain};
use vmax_voxcore::to_vmax_file as raw_to_vmax_file;

/// Writes a [`VoxelMaxVoxMain`] back to a [`VMaxFile`] with default
/// settings, the inverse of [`from_vmax_file`](crate::from_vmax_file). The
/// scene camera is the ext's when present, else the empty default;
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder) overrides it.
///
/// # Arguments
/// * `color_format` - where each palette's colors are stored.
pub fn to_vmax_file(
    state: &VoxelMaxVoxMain,
    color_format: VoxelMaxColorFormat,
) -> Result<VMaxFile> {
    Ok(raw_to_vmax_file(state, color_format)?)
}
