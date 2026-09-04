use crate::{MVoxVoxMain, Result};
use mvox_voxcore::codec::from_mvox_bytes as raw_from_mvox_bytes;

/// Loads the bytes of a MagicaVoxel `.vox` file into a [`MVoxVoxMain`].
/// The file is the one [`from_mvox_file`](crate::from_mvox_file) loads.
pub fn from_mvox_bytes(bytes: &[u8]) -> Result<MVoxVoxMain> {
    Ok(raw_from_mvox_bytes(bytes)?)
}
