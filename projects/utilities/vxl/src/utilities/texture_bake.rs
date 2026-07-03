use crate::ChannelPacking;

/// The lowered form of a `--texture` preset: what the bake writes.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureBake {
    /// The RGBA base color from the `rgba` attribute.
    RgbaColor,

    /// The emissive color: the `rgba` base color scaled by the `emissive`
    /// strength, for the glTF emissive slot.
    EmissiveColor,

    /// A scalar channel packing.
    Packing(ChannelPacking),
}
