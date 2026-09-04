use crate::{MVoxFile, MVoxVoxMain, Result};
use mvox_voxcore::from_mvox_file as raw_from_mvox_file;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a [`MVoxVoxMain`],
/// stashing the MagicaVoxel state with no native voxcore home in the ext.
pub fn from_mvox_file(file: &MVoxFile) -> Result<MVoxVoxMain> {
    Ok(raw_from_mvox_file(file)?)
}
