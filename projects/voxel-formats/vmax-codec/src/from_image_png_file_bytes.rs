use crate::{DecodePng, Error, Result};
use vmax::VMaxImage;

/// Decodes a `QuickLook/*.png` thumbnail into a [`VMaxImage`] through
/// `dependencies`: its dimensions and a row-major `[r, g, b, a]` pixel grid.
/// The inverse of
/// [`to_image_png_file_bytes`](crate::to_image_png_file_bytes).
pub fn from_image_png_file_bytes<D: DecodePng>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxImage> {
    dependencies.decode_png(bytes).map_err(Error::Png)
}
