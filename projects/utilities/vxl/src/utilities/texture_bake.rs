use crate::ChannelPacking;

/// The lowered form of a `--texture` preset: what the bake writes.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureBake {
    /// The RGBA base color from the `rgba` attribute.
    RgbaColor,
    /// A scalar channel packing.
    Packing(ChannelPacking),
}
