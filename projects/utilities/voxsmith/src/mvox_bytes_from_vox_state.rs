use crate::{Result, mvox_file_from_vox_state};
use mvox_codec::to_vox_file_bytes;
use voxcore::VoxState;

/// Writes a [`VoxState`] to the bytes of a MagicaVoxel `.vox` file, the inverse
/// of [`vox_state_from_mvox_bytes`](crate::vox_state_from_mvox_bytes). The state
/// is written back to an [`MVoxFile`](mvox::MVoxFile) and encoded with
/// [`mvox_codec`].
pub fn mvox_bytes_from_vox_state(state: &VoxState) -> Result<Vec<u8>> {
    let file = mvox_file_from_vox_state(state)?;
    Ok(to_vox_file_bytes(&file))
}
