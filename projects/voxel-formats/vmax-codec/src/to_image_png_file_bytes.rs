use crate::{EncodePng, Error, Result};
use vmax::VMaxImage;

/// Encodes a [`VMaxImage`] into `QuickLook/*.png` bytes through
/// `dependencies`: a `width x height` 8-bit RGBA PNG, one pixel per cell. The
/// inverse of [`from_image_png_file_bytes`](crate::from_image_png_file_bytes).
/// The round trip is pixel-lossless, not byte-identical, because the bytes
/// are re-encoded.
pub fn to_image_png_file_bytes<D: EncodePng>(
    dependencies: &D,
    image: &VMaxImage,
) -> Result<Vec<u8>> {
    let expected = (image.width as usize).saturating_mul(image.height as usize);
    if image.pixels.len() != expected {
        return Err(Error::Invalid(format!(
            "image has {} pixels but its {}x{} dimensions imply {expected}",
            image.pixels.len(),
            image.width,
            image.height
        )));
    }
    dependencies.encode_png(image).map_err(Error::Png)
}
