use crate::{Result, voxj_serde_file_from_vox_state};
use voxcore::VoxState;
use voxj_codec::to_voxj_file_bytes;

/// Writes a [`VoxState`] to compact `.voxj` JSON bytes, choosing the smallest
/// block encodings per object. The document is stamped with the current voxj
/// format version.
pub fn vox_state_to_voxj_bytes(state: &VoxState) -> Result<Vec<u8>> {
    let serde_file = voxj_serde_file_from_vox_state(state)?;
    Ok(to_voxj_file_bytes(&serde_file)?)
}
