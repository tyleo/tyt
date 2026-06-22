use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxPalettePngFile;

/// Encodes a [`VMaxPalettePngFile`] into `palette*.png` bytes: a `len x 1` RGBA PNG,
/// one pixel per color cell — the inverse of
/// [`from_palette_png_bytes`](crate::from_palette_png_bytes).
pub fn to_palette_png_bytes(file: &VMaxPalettePngFile) -> Result<Vec<u8>> {
    let pixels: Vec<u8> = file.0.iter().flatten().copied().collect();
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(&pixels, file.0.len() as u32, 1, ExtendedColorType::Rgba8)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    Ok(out)
}
