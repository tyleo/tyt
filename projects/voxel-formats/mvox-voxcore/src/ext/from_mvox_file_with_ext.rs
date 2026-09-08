use crate::{Result, ext::MVoxVoxMain, read_mvox};
use mvox::MVoxFile;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a [`MVoxVoxMain`] carrying
/// its [`MVoxExt`](crate::ext::MVoxExt), the typed form of
/// [`from_mvox_file`](crate::from_mvox_file). The state writes back exactly
/// through [`to_mvox_file_with_ext`](crate::ext::to_mvox_file_with_ext).
pub fn from_mvox_file_with_ext(file: &MVoxFile) -> Result<MVoxVoxMain> {
    let (state, mvox_ext) = read_mvox(file)?;

    Ok(state.map_ext(|()| Some(mvox_ext)))
}
