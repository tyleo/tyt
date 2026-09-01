use crate::{Result, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_voxcore::codec::to_voxjz_bytes as raw_to_voxjz_bytes;

/// Writes a [`VoxMain`] to a `.voxjz` zip archive holding one compact
/// `.voxj` member, with default settings. The document is the one
/// [`to_voxj_file`](crate::to_voxj_file) builds, and
/// [`from_voxj_bytes`](crate::from_voxj_bytes) reads it back.
pub fn to_voxjz_bytes<T: VoxExtSlot>(state: &VoxMain<T>) -> Result<Vec<u8>> {
    Ok(raw_to_voxjz_bytes(&VOXJ_DEPENDENCIES, state)?)
}
