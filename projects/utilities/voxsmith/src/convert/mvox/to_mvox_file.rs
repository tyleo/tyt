use crate::{MVoxFile, MVoxVoxMain, Result};
use mvox_voxcore::to_mvox_file as raw_to_mvox_file;

/// Writes a [`MVoxVoxMain`] back to a decoded MagicaVoxel [`MVoxFile`],
/// the inverse of [`from_mvox_file`](crate::from_mvox_file). A state without
/// an ext, such as one loaded from another format, has its file synthesized
/// from the bare scene.
pub fn to_mvox_file(state: &MVoxVoxMain) -> Result<MVoxFile> {
    Ok(raw_to_mvox_file(state)?)
}
