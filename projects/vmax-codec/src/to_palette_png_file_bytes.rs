use png::{BitDepth, ColorType, Encoder, Filter, SrgbRenderingIntent, chunk};
use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxPalettePngFile;

/// Static Exif block Voxel Max embeds in every `palette*.png`: a big-endian TIFF
/// whose Exif IFD records the sRGB color space plus the image dimensions. Bytes
/// `48..52` hold `PixelXDimension` and `60..64` hold `PixelYDimension`; the width
/// is patched per image in [`to_palette_png_file_bytes`] (the height is always 1).
const EXIF: [u8; 68] = [
    0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x87, 0x69, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xa0, 0x01, 0x00, 0x03,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xa0, 0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x01, 0x00, 0xa0, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00,
];

/// Encodes a [`VMaxPalettePngFile`] into `palette*.png` bytes: a `len x 1` RGBA PNG,
/// one pixel per color cell — the inverse of
/// [`from_palette_png_file_bytes`](crate::from_palette_png_file_bytes).
pub fn to_palette_png_file_bytes(file: &VMaxPalettePngFile) -> Result<Vec<u8>> {
    let pixels: Vec<u8> = file.0.iter().flatten().copied().collect();
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, file.0.len() as u32, 1);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    // Match Voxel Max's encoder: a Sub-filtered scanline tagged with an sRGB
    // chunk, followed by the Exif block restating the color space and dimensions.
    encoder.set_filter(Filter::Sub);
    encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
    let mut writer = encoder
        .write_header()
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    let mut exif = EXIF;
    exif[48..52].copy_from_slice(&(file.0.len() as u32).to_be_bytes());
    writer
        .write_chunk(chunk::eXIf, &exif)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    writer
        .write_image_data(&pixels)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    writer
        .finish()
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    Ok(out)
}
