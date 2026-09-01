use crate::{Result, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_voxcore::codec::from_voxj_bytes as raw_from_voxj_bytes;

/// Loads a `.voxj` or `.voxjz` document into a [`VoxMain`], typing the
/// document's `ext` block into the slot `T`. The container form is detected
/// from the leading bytes.
pub fn from_voxj_bytes<T: VoxExtSlot>(bytes: &[u8]) -> Result<VoxMain<T>> {
    Ok(raw_from_voxj_bytes(&VOXJ_DEPENDENCIES, bytes)?)
}
