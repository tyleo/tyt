use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxcore::VoxMain;
use voxsmith::{
    VoxjExtSlot, VoxjRawExt, VoxjRawVoxMain, from_goxl_bytes, from_mvox_bytes, from_qbcl_bytes,
    from_vmax_file, from_voxj_bytes,
};

/// Loads the voxel file at `input` into a [`VoxjRawVoxMain`] for the Voxel
/// Json writer. A Voxel Json input carries its document `ext` block verbatim,
/// so a re-encode keeps it whichever format owns it. Any other source encodes
/// its format ext into the block form here, the same block the Voxel Json
/// writer then persists.
pub fn load_state_voxj(input: &Path, from: Option<Format>) -> Result<VoxjRawVoxMain> {
    match implementation::resolve_format(input, from)? {
        Format::Voxj => Ok(from_voxj_bytes(&fs::read(input)?)?),
        Format::Goxl => raw_ext_state(from_goxl_bytes(&fs::read(input)?)?),
        Format::MVox => raw_ext_state(from_mvox_bytes(&fs::read(input)?)?),
        Format::Qbcl => raw_ext_state(from_qbcl_bytes(&fs::read(input)?)?),
        Format::VMax => raw_ext_state(from_vmax_file(&implementation::read_vmax_file(input)?)?),
    }
}

/// Re-types a format-typed state onto the verbatim block form, encoding its
/// slot through the format's [`VoxjExtSlot`].
fn raw_ext_state<T: VoxjExtSlot>(state: VoxMain<T>) -> Result<VoxjRawVoxMain> {
    let ext = state.ext().to_voxj_ext()?.map(VoxjRawExt);
    Ok(state.map_ext(|_| ext))
}
