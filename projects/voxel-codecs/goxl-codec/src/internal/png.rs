use crate::{Result, invalid};
use png::{BitDepth, ColorType, Decoder, Encoder};
use std::io::Cursor;

/// The edge length, in pixels, of a `BL16` block's `64 x 64` PNG. A block holds
/// [`GoxlBlock::SIZE`](goxl::GoxlBlock::SIZE)`^3 == 4096` voxels, one per pixel of
/// this `64 x 64` image.
pub const BLOCK_IMAGE_SIZE: u32 = 64;

/// Decodes an 8-bit `RGBA` PNG into its `(width, height, pixel bytes)`. Goxel
/// stores both the `BL16` voxel blocks and the `PREV` preview as 8-bit RGBA
/// PNGs, so a PNG of any other color type or bit depth is rejected. The returned
/// bytes are `width * height * 4`, four per pixel in image (row-major) order.
pub fn decode_png_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    let decoder = Decoder::new(Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|error| invalid(format!("invalid PNG: {error}")))?;

    let info = reader.info();
    let (color_type, bit_depth) = (info.color_type, info.bit_depth);
    if color_type != ColorType::Rgba || bit_depth != BitDepth::Eight {
        return Err(invalid(format!(
            "expected an 8-bit RGBA PNG, found {color_type:?} at {bit_depth:?}"
        )));
    }

    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| invalid("PNG dimensions too large".to_owned()))?;
    let mut buffer = vec![0u8; buffer_size];
    let frame = reader
        .next_frame(&mut buffer)
        .map_err(|error| invalid(format!("invalid PNG: {error}")))?;
    let (width, height) = (frame.width, frame.height);
    buffer.truncate(width as usize * height as usize * 4);
    Ok((width, height, buffer))
}

/// Encodes `width * height` 8-bit `RGBA` pixels into a PNG, the inverse of
/// [`decode_png_rgba`]. `rgba` must hold `width * height * 4` bytes, and `width`
/// and `height` must both be non-zero (PNG cannot encode a zero-area image).
pub fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("writing a PNG header to an in-memory buffer is infallible");
    writer
        .write_image_data(rgba)
        .expect("writing PNG image data to an in-memory buffer is infallible");
    writer
        .finish()
        .expect("finishing an in-memory PNG is infallible");
    out
}
