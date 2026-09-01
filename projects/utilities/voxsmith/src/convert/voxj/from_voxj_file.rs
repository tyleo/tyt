use crate::{Result, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::VoxjFile;
use voxj_voxcore::from_voxj_file as raw_from_voxj_file;

/// Loads a [`VoxjFile`] into a [`VoxMain`], typing the document's `ext`
/// block into the slot `T`.
pub fn from_voxj_file<T: VoxExtSlot>(file: &VoxjFile) -> Result<VoxMain<T>> {
    Ok(raw_from_voxj_file(&VOXJ_DEPENDENCIES, file)?)
}
