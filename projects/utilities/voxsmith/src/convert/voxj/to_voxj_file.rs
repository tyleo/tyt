use crate::{Result, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::VoxjFile;
use voxj_voxcore::to_voxj_file as raw_to_voxj_file;

/// Encodes a [`VoxMain`] into a [`VoxjFile`] with default settings, the
/// inverse of [`from_voxj_file`](crate::from_voxj_file). The slot rides the
/// `ext` block, each object takes its smallest block encodings, and the edit
/// state is recorded automatically.
/// [`VoxjFileBuilder`](crate::VoxjFileBuilder) configures those.
pub fn to_voxj_file<T: VoxExtSlot>(state: &VoxMain<T>) -> Result<VoxjFile> {
    Ok(raw_to_voxj_file(&VOXJ_DEPENDENCIES, state)?)
}
