use crate::{Error, Result};
use png::{BitDepth, ColorType, Encoder};
use vmax::VMaxImage;

/// Encodes a [`VMaxImage`] into `QuickLook/*.png` bytes: a `width x height`
/// 8-bit RGBA PNG, one pixel per cell. The inverse of
/// [`from_image_png_file_bytes`](crate::from_image_png_file_bytes). The bytes
/// are re-encoded rather than preserved, so the round-trip is pixel-lossless,
/// not byte-identical.
pub fn to_image_png_file_bytes(image: &VMaxImage) -> Result<Vec<u8>> {
    let expected = (image.width as usize).saturating_mul(image.height as usize);
    if image.pixels.len() != expected {
        return Err(Error::Invalid(format!(
            "image has {} pixels but its {}x{} dimensions imply {expected}",
            image.pixels.len(),
            image.width,
            image.height
        )));
    }
    let samples: Vec<u8> = image.pixels.iter().flatten().copied().collect();
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, image.width, image.height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&samples)?;
    writer.finish()?;
    Ok(out)
}
