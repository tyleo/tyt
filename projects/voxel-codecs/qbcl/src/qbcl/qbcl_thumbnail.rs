use crate::qbcl::QbclColor;

/// The preview image embedded in a `.qbcl` file's header: a `width` by `height`
/// grid of uncompressed pixels.
///
/// [`pixels`](Self::pixels) holds all `width * height` cells in row order (left to
/// right, top to bottom), so `(x, y)` is at index `x + width * y`. The pixels are
/// stored on disk as `BGRA`; they are normalized to [`QbclColor`]'s `RGBA` here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QbclThumbnail {
    /// Image width in pixels.
    pub width: u32,

    /// Image height in pixels.
    pub height: u32,

    /// Pixels, `width * height` of them in row order.
    pub pixels: Vec<QbclColor>,
}

impl QbclThumbnail {
    /// The pixel at `(x, y)`, or `None` if it is outside the image or the pixel
    /// buffer is not fully populated.
    pub fn pixel(&self, x: u32, y: u32) -> Option<QbclColor> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = x as usize + self.width as usize * y as usize;
        self.pixels.get(index).copied()
    }
}
