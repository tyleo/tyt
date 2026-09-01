use crate::{Result, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj_voxcore::codec::to_voxj_bytes as raw_to_voxj_bytes;

/// Writes a [`VoxMain`] to compact `.voxj` JSON bytes with default settings,
/// the inverse of [`from_voxj_bytes`](crate::from_voxj_bytes). The document
/// is the one [`to_voxj_file`](crate::to_voxj_file) builds.
pub fn to_voxj_bytes<T: VoxExtSlot>(state: &VoxMain<T>) -> Result<Vec<u8>> {
    Ok(raw_to_voxj_bytes(&VOXJ_DEPENDENCIES, state)?)
}
