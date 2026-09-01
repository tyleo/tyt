use crate::{EditStateMode, Result, write_voxj};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::VoxjFile;

/// Encodes a [`VoxMain`] into a [`VoxjFile`], choosing the smallest
/// per-object block encodings and persisting the slot's `ext` block. The
/// canonical shipping form, and the body behind the `.voxj` and `.voxjz`
/// writers. For control over the block encodings, the ext block, or the edit
/// state, use [`VoxjFileBuilder`](crate::VoxjFileBuilder).
pub fn to_voxj_file<T: VoxExtSlot>(state: &VoxMain<T>) -> Result<VoxjFile> {
    write_voxj(state, None, None, true, EditStateMode::Auto)
}
