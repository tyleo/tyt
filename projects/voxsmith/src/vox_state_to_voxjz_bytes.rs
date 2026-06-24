use crate::voxj_serde_file_from_vox_state;
use std::io;
use voxcore::VoxState;
use voxj_codec::to_voxjz_file_bytes;

/// Writes a [`VoxState`] to a `.voxjz` zip archive holding one compact `.voxj`
/// member, choosing the smallest block encodings per object. The document is
/// stamped with the current voxj format version.
pub fn vox_state_to_voxjz_bytes(state: &VoxState) -> io::Result<Vec<u8>> {
    let serde_file = voxj_serde_file_from_vox_state(state)?;
    to_voxjz_file_bytes(&serde_file).map_err(io::Error::other)
}
