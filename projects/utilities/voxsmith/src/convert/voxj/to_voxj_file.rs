use crate::{EditStateMode, Result, VoxjExtSlot, write_voxj};
use voxcore::VoxMain;
use voxj::VoxjFile;

/// Encodes a [`VoxMain`] into a [`VoxjFile`], choosing the smallest per-object
/// block encodings. The canonical shipping form, and the body behind the
/// `.voxj` and `.voxjz` writers. For control over the block encodings, the ext
/// block, or the edit state, use [`VoxjFileBuilder`](crate::VoxjFileBuilder).
pub fn to_voxj_file<T: VoxjExtSlot>(state: &VoxMain<T>) -> Result<VoxjFile> {
    write_voxj(state, None, None, true, EditStateMode::Auto)
}
