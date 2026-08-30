/// One pixel of a [`QbclThumbnail`](crate::qbcl::QbclThumbnail): an `RGBA` color,
/// one byte per channel.
///
/// The thumbnail is stored on disk as uncompressed `BGRA` pixels; channels are
/// normalized to `RGBA` in memory, so the on-disk order is undone on read and
/// reapplied on write.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct QbclColor {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Alpha channel.
    pub a: u8,
}

impl QbclColor {
    /// A color from its four channels.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
