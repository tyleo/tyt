use crate::{GOXL_DEPENDENCIES, GoxelVoxMain, Result};
use goxl_voxcore::codec::from_goxl_bytes as raw_from_goxl_bytes;

/// Loads the bytes of a Goxel `.gox` file into a [`GoxelVoxMain`]. The
/// state is the one [`from_goxl_file`](crate::from_goxl_file) loads.
pub fn from_goxl_bytes(bytes: &[u8]) -> Result<GoxelVoxMain> {
    Ok(raw_from_goxl_bytes(&GOXL_DEPENDENCIES, bytes)?)
}
