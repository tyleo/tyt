/// A straight-alpha RGBA color, one byte per channel, as stored in an `RGBA`
/// chunk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MVoxColor {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Alpha channel (opacity).
    pub a: u8,
}

impl MVoxColor {
    /// A color from its four channels.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
