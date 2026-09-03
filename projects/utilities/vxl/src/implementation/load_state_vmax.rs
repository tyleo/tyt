use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxsmith::{VoxelMaxVoxMain, from_voxj_bytes};

/// Loads the voxel file at `input` into a [`VoxelMaxVoxMain`] for the Voxel
/// Max writer. A Voxel Max input keeps its ext. A Voxel Json input reads the
/// ext back from its document `ext` block when that block is the Voxel Max
/// one. Every other source, including a Voxel Json document with a foreign
/// block, loads with no ext, so the writer synthesizes from the bare scene.
pub fn load_state_vmax(input: &Path, from: Option<Format>) -> Result<VoxelMaxVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::VMax => implementation::load_vmax_package(input),
        Format::Voxj => Ok(from_voxj_bytes(&fs::read(input)?)?),
        format => Ok(implementation::load_state(input, Some(format))?.map_ext(|_| None)),
    }
}
