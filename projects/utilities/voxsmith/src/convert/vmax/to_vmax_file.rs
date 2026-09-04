use crate::{Result, VMaxColorFormat, VMaxFile, VMaxVoxMain};
use vmax_voxcore::to_vmax_file as raw_to_vmax_file;

/// Writes a [`VMaxVoxMain`] back to a [`VMaxFile`] with default
/// settings, the inverse of [`from_vmax_file`](crate::from_vmax_file). The
/// scene camera is the ext's when present, else the empty default;
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder) overrides it.
///
/// # Arguments
/// * `color_format` - where each palette's colors are stored.
pub fn to_vmax_file(state: &VMaxVoxMain, color_format: VMaxColorFormat) -> Result<VMaxFile> {
    Ok(raw_to_vmax_file(state, color_format)?)
}
