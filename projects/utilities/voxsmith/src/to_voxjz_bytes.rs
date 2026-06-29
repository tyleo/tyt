use crate::{Result, to_voxj_file};
use voxcore::VoxMain;
use voxj_codec::to_voxjz_file_bytes;

/// Writes a [`VoxMain`] to a `.voxjz` zip archive holding one compact `.voxj`
/// member, choosing the smallest block encodings per object. The document is
/// stamped with the current voxj format version.
pub fn to_voxjz_bytes(state: &VoxMain) -> Result<Vec<u8>> {
    let file = to_voxj_file(state)?;
    Ok(to_voxjz_file_bytes(&file)?)
}
