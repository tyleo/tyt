use crate::{Result, to_voxj_file_with_encoding};
use voxcore::VoxMain;
use voxj::VoxjFile;

/// Encodes a [`VoxMain`] into a [`VoxjFile`], choosing the smallest per-object
/// block encodings. The canonical shipping form, and the body behind the `.voxj`
/// and `.voxjz` writers. For a fixed encoding instead, see
/// [`to_voxj_file_with`](crate::to_voxj_file_with).
pub fn to_voxj_file(state: &VoxMain) -> Result<VoxjFile> {
    to_voxj_file_with_encoding(state, None)
}
