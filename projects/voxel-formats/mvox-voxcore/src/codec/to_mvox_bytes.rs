use crate::{MagicaVoxelVoxMain, Result, to_mvox_file};
use mvox_codec::to_mvox_file_bytes;

/// Writes a [`MagicaVoxelVoxMain`] to the bytes of a MagicaVoxel `.vox` file,
/// the bytes form of [`to_mvox_file`] and the inverse of
/// [`from_mvox_bytes`](crate::codec::from_mvox_bytes).
pub fn to_mvox_bytes(state: &MagicaVoxelVoxMain) -> Result<Vec<u8>> {
    let file = to_mvox_file(state)?;
    Ok(to_mvox_file_bytes(&file))
}
