use std::{io::Result, path::Path};

/// Loads an image from disk and converts it to RGBA8, returning the pixel data, width, and height.
pub fn load_image_rgba(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
    let img = image::open(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let rgba = img.into_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Ok((rgba.into_raw(), w, h))
}
