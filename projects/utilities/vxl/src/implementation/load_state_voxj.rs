use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxsmith::{
    VoxjVoxMain, from_goxl_bytes, from_mvox_bytes, from_qbcl_bytes, from_voxj_bytes,
    to_voxj_vox_main,
};

/// Loads the voxel file at `input` into a [`VoxjVoxMain`] for the Voxel
/// Json writer. A Voxel Json input carries its document `ext` block verbatim,
/// so a re-encode keeps it whichever format owns it. Any other source encodes
/// its format ext into the block form here, the same block the Voxel Json
/// writer then persists.
pub fn load_state_voxj(input: &Path, from: Option<Format>) -> Result<VoxjVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::Voxj => Ok(from_voxj_bytes(&fs::read(input)?)?),
        Format::Goxl => Ok(to_voxj_vox_main(from_goxl_bytes(&fs::read(input)?)?)?),
        Format::MVox => Ok(to_voxj_vox_main(from_mvox_bytes(&fs::read(input)?)?)?),
        Format::Qbcl => Ok(to_voxj_vox_main(from_qbcl_bytes(&fs::read(input)?)?)?),
        Format::VMax => Ok(to_voxj_vox_main(implementation::load_vmax_package(input)?)?),
    }
}
