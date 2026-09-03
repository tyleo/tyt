use vmax::{VMaxImage, VMaxPalettePngFile};

/// Encodes PNGs, the form of the `palette*.png` color tables and the
/// `QuickLook/` thumbnails.
pub trait EncodePng {
    /// A `width x height` 8-bit RGBA PNG of `image`, one pixel per cell, or
    /// the reason it cannot be encoded. `image` holds `width * height` pixels.
    fn encode_png(&self, image: &VMaxImage) -> Result<Vec<u8>, String>;

    /// A `len x 1` 8-bit RGBA PNG of `file`'s colors in the shape Voxel Max's
    /// encoder writes, or the reason it cannot be encoded. That shape is a
    /// Sub-filtered scanline tagged with an sRGB chunk, then an Exif block
    /// restating the color space and dimensions.
    fn encode_palette_png(&self, file: &VMaxPalettePngFile) -> Result<Vec<u8>, String>;
}
