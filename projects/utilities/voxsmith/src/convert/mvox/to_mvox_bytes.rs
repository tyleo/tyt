use crate::{MagicaVoxelVoxMain, Result};
use mvox_voxcore::codec::to_mvox_bytes as raw_to_mvox_bytes;

/// Writes a [`MagicaVoxelVoxMain`] to the bytes of a MagicaVoxel `.vox` file,
/// the inverse of [`from_mvox_bytes`](crate::from_mvox_bytes). The file is the
/// one [`to_mvox_file`](crate::to_mvox_file) builds.
pub fn to_mvox_bytes(state: &MagicaVoxelVoxMain) -> Result<Vec<u8>> {
    Ok(raw_to_mvox_bytes(state)?)
}
