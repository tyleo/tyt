use crate::{Result, VoxelMaxVoxMain};
use std::io::Result as IOResult;
use vmax_voxcore::codec::from_vmax_package as raw_from_vmax_package;

/// Loads a `.vmax` package into a [`VoxelMaxVoxMain`], reading its files
/// through the caller's closures. The document is the one
/// [`from_vmax_file`](crate::from_vmax_file) loads.
///
/// # Arguments
/// * `list` - returns the package-relative path of every file, so `QuickLook/`
///   entries keep their subdirectory prefix.
/// * `resolve` - returns a file's bytes by that path, or `Ok(None)` if it has
///   since vanished.
pub fn from_vmax_package<L, R>(list: L, mut resolve: R) -> Result<VoxelMaxVoxMain>
where
    L: FnOnce() -> IOResult<Vec<String>>,
    R: FnMut(&str) -> IOResult<Option<Vec<u8>>>,
{
    Ok(raw_from_vmax_package(
        || list().map_err(Into::into),
        |name| resolve(name).map_err(Into::into),
    )?)
}
