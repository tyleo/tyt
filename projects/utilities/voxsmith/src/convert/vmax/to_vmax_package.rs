use crate::{Result, VMAX_DEPENDENCIES, VMaxColorFormat, VMaxVoxMain};
use std::io::Result as IOResult;
use vmax_voxcore::codec::to_vmax_package as raw_to_vmax_package;

/// Writes a [`VMaxVoxMain`] to a `.vmax` package with default settings,
/// the inverse of [`from_vmax_package`](crate::from_vmax_package), emitting
/// each file through the caller's closure. The document is the one
/// [`to_vmax_file`](crate::to_vmax_file) builds.
///
/// # Arguments
/// * `color_format` - where each palette's colors are stored.
/// * `write` - receives each file's package-relative name and bytes and
///   performs the actual write, creating any subdirectory a `QuickLook/` name
///   implies.
pub fn to_vmax_package<W>(
    state: &VMaxVoxMain,
    color_format: VMaxColorFormat,
    mut write: W,
) -> Result<()>
where
    W: FnMut(&str, &[u8]) -> IOResult<()>,
{
    Ok(raw_to_vmax_package(
        &VMAX_DEPENDENCIES,
        state,
        color_format,
        |name, bytes| write(name, bytes).map_err(Into::into),
    )?)
}
