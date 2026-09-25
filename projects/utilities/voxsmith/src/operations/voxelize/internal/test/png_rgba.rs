use png::{BitDepth, ColorType, Encoder};

/// Encodes `texels` (row-major RGBA8) into a PNG of the given size.
pub(crate) fn png_rgba(width: u32, height: u32, texels: &[[u8; 4]]) -> Vec<u8> {
    let samples: Vec<u8> = texels.iter().flatten().copied().collect();
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&samples).unwrap();
    writer.finish().unwrap();
    bytes
}
