use crate::{Error, Result};

/// Encodes an RGBA8 pixel buffer as PNG bytes. `pixels` is `width * height * 4`
/// bytes, row-major from the top-left.
pub(crate) fn encode_rgba8_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();

    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().map_err(Error::invalid)?;
        writer.write_image_data(pixels).map_err(Error::invalid)?;
    }

    Ok(bytes)
}
