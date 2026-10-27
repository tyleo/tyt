use crate::{DecodePng, Error, Result};
use vmax::VMaxPalettePngFile;

/// Decodes `palette*.png` bytes into a [`VMaxPalettePngFile`] color table (one
/// `[r, g, b, a]` cell per pixel, in image order) through `dependencies`.
pub fn from_palette_png_file_bytes<D: DecodePng>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxPalettePngFile> {
    let image = dependencies.decode_png(bytes).map_err(Error::Png)?;
    Ok(VMaxPalettePngFile(image.pixels))
}
