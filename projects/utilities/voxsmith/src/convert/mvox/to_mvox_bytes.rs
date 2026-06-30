use crate::{Result, to_mvox_file};
use mvox_codec::to_mvox_file_bytes;
use voxcore::VoxMain;

/// Writes a [`VoxMain`] to the bytes of a MagicaVoxel `.vox` file, the inverse
/// of [`from_mvox_bytes`](crate::from_mvox_bytes). The state
/// is written back to an [`MVoxFile`](mvox::MVoxFile) and encoded with
/// [`mvox_codec`].
pub fn to_mvox_bytes(state: &VoxMain) -> Result<Vec<u8>> {
    let file = to_mvox_file(state)?;
    Ok(to_mvox_file_bytes(&file))
}
