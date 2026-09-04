use crate::{MagicaVoxelVoxMain, Result, from_mvox_file};
use mvox_codec::from_mvox_file_bytes;

/// Loads the bytes of a MagicaVoxel `.vox` file into a [`MagicaVoxelVoxMain`],
/// the bytes form of [`from_mvox_file`].
pub fn from_mvox_bytes(bytes: &[u8]) -> Result<MagicaVoxelVoxMain> {
    let file = from_mvox_file_bytes(bytes)?;
    from_mvox_file(&file)
}
