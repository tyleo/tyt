use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxcore::VoxMain;
use voxj_voxcore::codec::from_voxj_bytes;
use voxsmith::{from_goxl_bytes, from_mvox_bytes, from_qbcl_bytes, from_vmax_file};

/// Loads the voxel file at `input` into a [`VoxMain`] carrying no ext, the
/// front half of every read-only command. `from` picks the source format,
/// inferred from `input`'s extension when `None`. Any format ext the source
/// carries is dropped here. A command that writes the state back to a format
/// loads through that format's typed loader instead, so its ext survives.
pub fn load_state(input: &Path, from: Option<Format>) -> Result<VoxMain> {
    let state = match implementation::resolve_format(input, from)? {
        Format::Voxj => from_voxj_bytes(&fs::read(input)?)?,
        Format::MVox => from_mvox_bytes(&fs::read(input)?)?.map_ext(|_| ()),
        Format::Goxl => from_goxl_bytes(&fs::read(input)?)?.map_ext(|_| ()),
        Format::Qbcl => from_qbcl_bytes(&fs::read(input)?)?.map_ext(|_| ()),
        Format::VMax => from_vmax_file(&implementation::read_vmax_file(input)?)?.map_ext(|_| ()),
    };
    Ok(state)
}
