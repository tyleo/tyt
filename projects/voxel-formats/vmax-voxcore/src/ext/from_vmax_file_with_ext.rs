use crate::{Result, ext::VMaxVoxMain, read_vmax};
use vmax::VMaxFile;

/// Loads a Voxel Max document into a [`VMaxVoxMain`] carrying its
/// [`VMaxExt`](crate::ext::VMaxExt), the typed form of
/// [`from_vmax_file`](crate::from_vmax_file). The state writes back exactly
/// through [`to_vmax_file_with_ext`](crate::ext::to_vmax_file_with_ext).
pub fn from_vmax_file_with_ext(serde: &VMaxFile) -> Result<VMaxVoxMain> {
    read_vmax(serde)
}
