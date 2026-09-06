use vmax::VMaxImage;

/// Decodes a PNG, the form of the `palette*.png` color tables and the
/// `QuickLook/` thumbnails.
pub trait DecodePng {
    /// The image `bytes` hold as 8-bit RGBA, or the reason they are not one.
    /// Gray, paletted, sub-8-bit, and 16-bit inputs normalize to one
    /// `[r, g, b, a]` cell per pixel.
    fn decode_png(&self, bytes: &[u8]) -> Result<VMaxImage, String>;
}
