use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxPalettePngFile;

/// Decodes `palette*.png` bytes into a [`VMaxPalettePngFile`] color table (one
/// `[r, g, b, a]` cell per pixel, in image order).
pub fn from_palette_png_bytes(bytes: &[u8]) -> Result<VMaxPalettePngFile> {
    let image =
        image::load_from_memory(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    let colors = image
        .into_rgba8()
        .into_raw()
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect();
    Ok(VMaxPalettePngFile(colors))
}
