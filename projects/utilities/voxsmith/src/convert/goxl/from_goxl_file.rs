use crate::{GoxlFile, GoxlVoxMain, Result};
use goxl_voxcore::from_goxl_file as raw_from_goxl_file;

/// Loads a decoded Goxel [`GoxlFile`] into a [`GoxlVoxMain`], stashing the
/// Goxel state with no native voxcore home in the ext.
pub fn from_goxl_file(file: &GoxlFile) -> Result<GoxlVoxMain> {
    Ok(raw_from_goxl_file(file)?)
}
