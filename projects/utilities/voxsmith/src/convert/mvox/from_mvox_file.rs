use crate::{MVoxFile, MagicaVoxelVoxMain, Result};
use mvox_voxcore::from_mvox_file as raw_from_mvox_file;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a [`MagicaVoxelVoxMain`],
/// stashing the MagicaVoxel state with no native voxcore home in the ext.
pub fn from_mvox_file(file: &MVoxFile) -> Result<MagicaVoxelVoxMain> {
    Ok(raw_from_mvox_file(file)?)
}
