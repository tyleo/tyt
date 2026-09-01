use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxj::DependenciesImpl as VoxjDependenciesImpl;
use voxj_voxcore::codec::from_voxj_bytes;
use voxsmith::{GoxelVoxMain, from_goxl_bytes};

/// Loads the voxel file at `input` into a [`GoxelVoxMain`] for the Goxel
/// `.gox` writer. A Goxel `.gox` input keeps its ext. A Voxel Json input
/// reads the ext back from its document `ext` block when that block is the
/// Goxel `.gox` one. Every other source, including a Voxel Json document with
/// a foreign block, loads with no ext, so the writer synthesizes from the
/// bare scene.
pub fn load_state_goxl(input: &Path, from: Option<Format>) -> Result<GoxelVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::Goxl => Ok(from_goxl_bytes(&fs::read(input)?)?),
        Format::Voxj => Ok(from_voxj_bytes(&VoxjDependenciesImpl, &fs::read(input)?)?),
        format => Ok(implementation::load_state(input, Some(format))?.map_ext(|_| None)),
    }
}
