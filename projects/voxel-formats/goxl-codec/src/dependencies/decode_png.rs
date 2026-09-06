use crate::GoxlRgbaImage;

/// Decodes a PNG, the form of the `BL16` voxel blocks and the `PREV` preview.
pub trait DecodePng {
    /// The 8-bit RGBA image `bytes` hold, or the reason they are not one.
    fn decode_png(&self, bytes: &[u8]) -> Result<GoxlRgbaImage, String>;
}
