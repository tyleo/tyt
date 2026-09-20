use crate::dependencies::mesh::PngImage;

/// Encodes PNGs, the form of the baked images.
pub trait EncodePng {
    /// An 8-bit PNG of `image` in its channel format, its transfer stamped in
    /// the color chunks, or the reason it cannot be encoded.
    fn encode_png(&self, image: &PngImage) -> Result<Vec<u8>, String>;
}
