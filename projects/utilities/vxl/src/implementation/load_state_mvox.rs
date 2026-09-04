use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxsmith::{MVoxVoxMain, from_mvox_bytes, from_voxj_bytes};

/// Loads the voxel file at `input` into a [`MVoxVoxMain`] for the
/// MagicaVoxel `.vox` writer. A MagicaVoxel `.vox` input keeps its ext. A
/// Voxel Json input reads the ext back from its document `ext` block when
/// that block is the MagicaVoxel `.vox` one. Every other source, including a
/// Voxel Json document with a foreign block, loads with no ext, so the writer
/// synthesizes from the bare scene.
pub fn load_state_mvox(input: &Path, from: Option<Format>) -> Result<MVoxVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::MVox => Ok(from_mvox_bytes(&fs::read(input)?)?),
        Format::Voxj => Ok(from_voxj_bytes(&fs::read(input)?)?),
        format => Ok(implementation::load_state(input, Some(format))?.map_ext(|_| None)),
    }
}
