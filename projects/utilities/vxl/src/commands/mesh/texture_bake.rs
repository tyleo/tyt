use crate::commands::ChannelPacking;

/// The lowered form of a `--texture` preset: what the bake writes.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureBake {
    /// The RGBA base color from the `baseColor` property.
    RgbaColor,

    /// The emissive color: `emissiveColor` scaled by `emissiveStrength`, for
    /// the glTF emissive slot.
    EmissiveColor,

    /// A scalar channel packing.
    Packing(ChannelPacking),
}
