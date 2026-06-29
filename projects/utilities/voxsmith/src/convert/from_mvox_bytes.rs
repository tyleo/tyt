use crate::{Result, from_mvox_file};
use mvox_codec::from_mvox_file_bytes;
use voxcore::VoxMain;

/// Loads the bytes of a MagicaVoxel `.vox` file into a [`VoxMain`]. The bytes
/// are decoded with [`mvox_codec`] and the result is loaded into voxcore's
/// in-memory form.
pub fn from_mvox_bytes(bytes: &[u8]) -> Result<VoxMain> {
    let file = from_mvox_file_bytes(bytes)?;
    from_mvox_file(&file)
}
