use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxj::DependenciesImpl as VoxjDependenciesImpl;
use voxj_voxcore::{VoxjVoxMain, codec::from_voxj_bytes};
use voxsmith::{from_goxl_bytes, from_mvox_bytes, from_qbcl_bytes, from_vmax_file};

/// Loads the voxel file at `input` into a [`VoxjVoxMain`] for the Voxel
/// Json writer. A Voxel Json input carries its document `ext` block verbatim,
/// so a re-encode keeps it whichever format owns it. Any other source encodes
/// its format ext into the block form here, the same block the Voxel Json
/// writer then persists.
pub fn load_state_voxj(input: &Path, from: Option<Format>) -> Result<VoxjVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::Voxj => Ok(from_voxj_bytes(&VoxjDependenciesImpl, &fs::read(input)?)?),
        Format::Goxl => implementation::raw_ext_state(from_goxl_bytes(&fs::read(input)?)?),
        Format::MVox => implementation::raw_ext_state(from_mvox_bytes(&fs::read(input)?)?),
        Format::Qbcl => implementation::raw_ext_state(from_qbcl_bytes(&fs::read(input)?)?),
        Format::VMax => {
            implementation::raw_ext_state(from_vmax_file(&implementation::read_vmax_file(input)?)?)
        }
    }
}
