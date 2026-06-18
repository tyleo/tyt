use std::io::{Error as IOError, ErrorKind, Result};

/// Decodes an in-memory image to RGBA8, returning the pixel data, width, and height.
pub fn decode_image_rgba(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    let img =
        image::load_from_memory(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Ok((rgba.into_raw(), w, h))
}
