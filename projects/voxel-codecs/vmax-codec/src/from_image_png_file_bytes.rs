use crate::{Error, Result};
use png::{Decoder, Transformations};
use std::io::Cursor;
use vmax::VMaxImage;

/// Decodes a `QuickLook/*.png` thumbnail into a [`VMaxImage`]: its dimensions
/// and a row-major `[r, g, b, a]` pixel grid. The inverse of
/// [`to_image_png_file_bytes`](crate::to_image_png_file_bytes).
pub fn from_image_png_file_bytes(bytes: &[u8]) -> Result<VMaxImage> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    // Normalize paletted, sub-8-bit, and 16-bit inputs down to 8-bit channels so
    // each pixel reduces to a single `[r, g, b, a]` cell below.
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| Error::Invalid("png dimensions too large".to_owned()))?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer)?;
    let pixels = buffer[..info.buffer_size()]
        .chunks_exact(info.color_type.samples())
        .map(|cell| match cell {
            [gray] => [*gray, *gray, *gray, u8::MAX],
            [gray, alpha] => [*gray, *gray, *gray, *alpha],
            [r, g, b] => [*r, *g, *b, u8::MAX],
            [r, g, b, alpha] => [*r, *g, *b, *alpha],
            _ => [0, 0, 0, u8::MAX],
        })
        .collect();
    Ok(VMaxImage {
        width: info.width,
        height: info.height,
        pixels,
    })
}
