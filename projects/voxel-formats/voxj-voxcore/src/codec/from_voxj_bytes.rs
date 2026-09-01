use crate::{Result, from_voxj_file};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_codec::from_voxj_or_voxjz_file_bytes;

/// Loads a `.voxj` or `.voxjz` document into a [`VoxMain`], typing the
/// document's `ext` block into the slot `T`. The container form is detected
/// from the leading bytes, then each object's encoded geometry is decoded
/// and the result is loaded into voxcore's in-memory form.
pub fn from_voxj_bytes<T: VoxExtSlot>(bytes: &[u8]) -> Result<VoxMain<T>> {
    let file = from_voxj_or_voxjz_file_bytes(bytes)?;
    from_voxj_file(&file)
}
