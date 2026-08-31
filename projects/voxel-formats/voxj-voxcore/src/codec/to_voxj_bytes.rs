use crate::{Result, VoxjVoxMain, to_voxj_file};
use voxj_codec::to_voxj_file_bytes;

/// Writes a [`VoxjVoxMain`] to compact `.voxj` JSON bytes, choosing the
/// smallest block encodings per object. The document is stamped with the
/// current voxj format version.
pub fn to_voxj_bytes(state: &VoxjVoxMain) -> Result<Vec<u8>> {
    let file = to_voxj_file(state)?;
    Ok(to_voxj_file_bytes(&file)?)
}
