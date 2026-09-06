use crate::GoxlRgbaImage;

/// Encodes a PNG, the form of the `BL16` voxel blocks and the `PREV` preview.
pub trait EncodePng {
    /// A `width x height` 8-bit RGBA PNG of `image`, one pixel per cell. The
    /// writer only passes an `image` with non-zero dimensions holding
    /// `width * height` pixels.
    fn encode_png(&self, image: &GoxlRgbaImage) -> Vec<u8>;
}
