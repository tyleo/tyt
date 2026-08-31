use crate::{Result, VoxjExtSlot};
use voxcore::VoxMain;
use voxj_voxcore::codec::from_voxj_bytes as raw_from_voxj_bytes;

/// Loads a `.voxj` or `.voxjz` document into a [`VoxMain`], typing the
/// document's `ext` block into the slot `T`. The container form is detected
/// from the leading bytes.
pub fn from_voxj_bytes<T: VoxjExtSlot>(bytes: &[u8]) -> Result<VoxMain<T>> {
    let state = raw_from_voxj_bytes(bytes)?;
    let ext = T::from_voxj_ext(state.ext().as_ref())?;
    Ok(state.map_ext(|_| ext))
}
