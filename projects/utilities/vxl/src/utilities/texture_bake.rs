use crate::ChannelPacking;

/// The lowered form of a `--texture` preset: what the bake writes. Most presets
/// are a scalar [`ChannelPacking`]; `albedo` is the RGBA base color read from the
/// `rgba` attribute, which a scalar packing cannot express.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureBake {
    /// The RGBA base color from the `rgba` attribute.
    RgbaColor,
    /// A scalar channel packing.
    Packing(ChannelPacking),
}
