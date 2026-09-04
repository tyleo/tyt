use crate::{GoxlFile, GoxlVoxMain, Result};
use goxl_voxcore::to_goxl_file as raw_to_goxl_file;

/// Writes a [`GoxlVoxMain`] back to a decoded Goxel [`GoxlFile`], the
/// inverse of [`from_goxl_file`](crate::from_goxl_file). A state without an
/// ext has its file synthesized from the bare scene.
pub fn to_goxl_file(state: &GoxlVoxMain) -> Result<GoxlFile> {
    Ok(raw_to_goxl_file(state)?)
}
