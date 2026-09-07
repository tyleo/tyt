use crate::operations::mesh::AtlasImage;

/// Encodes PNGs, the form of the baked material atlas images.
pub trait EncodePng {
    /// A `width x height` 8-bit RGBA PNG of `image`, one pixel per cell, or
    /// the reason it cannot be encoded.
    fn encode_png(&self, image: &AtlasImage) -> Result<Vec<u8>, String>;
}
