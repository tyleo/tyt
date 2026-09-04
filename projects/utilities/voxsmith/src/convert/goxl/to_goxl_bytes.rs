use crate::{GOXL_DEPENDENCIES, GoxelVoxMain, Result};
use goxl_voxcore::codec::to_goxl_bytes as raw_to_goxl_bytes;

/// Writes a [`GoxelVoxMain`] to the bytes of a Goxel `.gox` file, the
/// inverse of [`from_goxl_bytes`](crate::from_goxl_bytes). The file is the
/// one [`to_goxl_file`](crate::to_goxl_file) builds.
pub fn to_goxl_bytes(state: &GoxelVoxMain) -> Result<Vec<u8>> {
    Ok(raw_to_goxl_bytes(&GOXL_DEPENDENCIES, state)?)
}
