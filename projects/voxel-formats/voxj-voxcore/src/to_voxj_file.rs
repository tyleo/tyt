use crate::{EditStateMode, Result, write_voxj};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};
use voxj::{CostVoxjObject, EncodeBase64, VoxjFile};

/// Encodes a [`VoxMain`] into a [`VoxjFile`], choosing each object's block
/// encodings by the lowest cost and persisting the state's `ext` block. The
/// canonical shipping form under the deflated cost of
/// `voxj::DependenciesImpl`, and the body behind the `.voxj` and `.voxjz`
/// writers. For control over the block encodings, the ext block, or the edit
/// state, use [`VoxjFileBuilder`](crate::VoxjFileBuilder).
pub fn to_voxj_file<T: VoxExtBlockCodec, D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    state: &VoxMain<T>,
) -> Result<VoxjFile> {
    write_voxj(dependencies, state, None, None, true, EditStateMode::Auto)
}
