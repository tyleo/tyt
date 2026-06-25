use crate::{Result, vox_state_from_mvox_file};
use mvox_codec::from_vox_file_bytes;
use voxcore::VoxState;

/// Loads the bytes of a MagicaVoxel `.vox` file into a [`VoxState`]. The bytes
/// are decoded with [`mvox_codec`] and the result is loaded into voxcore's
/// in-memory form.
pub fn vox_state_from_mvox_bytes(bytes: &[u8]) -> Result<VoxState> {
    let file = from_vox_file_bytes(bytes)?;
    vox_state_from_mvox_file(&file)
}
