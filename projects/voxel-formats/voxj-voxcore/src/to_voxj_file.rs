use crate::{EditStateMode, Result, VoxjVoxMain, write_voxj};
use voxj::VoxjFile;

/// Encodes a [`VoxjVoxMain`] into a [`VoxjFile`], choosing the smallest
/// per-object block encodings and keeping the carried `ext` block. The
/// canonical shipping form, and the body behind the `.voxj` and `.voxjz`
/// writers. For control over the block encodings, the ext block, or the edit
/// state, use [`VoxjFileBuilder`](crate::VoxjFileBuilder).
pub fn to_voxj_file(state: &VoxjVoxMain) -> Result<VoxjFile> {
    write_voxj(state, None, None, true, EditStateMode::Auto)
}
