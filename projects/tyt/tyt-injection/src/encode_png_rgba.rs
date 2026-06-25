use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use std::io::{Error as IOError, ErrorKind, Result};

/// Encodes RGBA8 pixel data (`width * height * 4` bytes) into PNG bytes — the
/// inverse of [`decode_image_rgba`](crate::decode_image_rgba).
pub fn encode_png_rgba(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    Ok(out)
}
