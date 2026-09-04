use crate::{GOXL_DEPENDENCIES, GoxlVoxMain, Result};
use goxl_voxcore::codec::from_goxl_bytes as raw_from_goxl_bytes;

/// Loads the bytes of a Goxel `.gox` file into a [`GoxlVoxMain`]. The
/// state is the one [`from_goxl_file`](crate::from_goxl_file) loads.
pub fn from_goxl_bytes(bytes: &[u8]) -> Result<GoxlVoxMain> {
    Ok(raw_from_goxl_bytes(&GOXL_DEPENDENCIES, bytes)?)
}
