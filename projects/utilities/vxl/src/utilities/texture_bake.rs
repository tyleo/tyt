use crate::ChannelPacking;

/// The lowered form of a `--texture` preset: what the bake writes.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureBake {
    /// The RGBA base color from the `baseColorFactor` attribute.
    RgbaColor,

    /// The emissive color: `emissiveFactor` scaled by `emissiveStrength`, for
    /// the glTF emissive slot.
    EmissiveColor,

    /// A scalar channel packing.
    Packing(ChannelPacking),
}
