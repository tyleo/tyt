use crate::{GoxelVoxMain, GoxlFile, Result};
use goxl_voxcore::from_goxl_file as raw_from_goxl_file;

/// Loads a decoded Goxel [`GoxlFile`] into a [`GoxelVoxMain`], stashing the
/// Goxel state with no native voxcore home in the ext.
pub fn from_goxl_file(file: &GoxlFile) -> Result<GoxelVoxMain> {
    Ok(raw_from_goxl_file(file)?)
}
