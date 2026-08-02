use crate::MaterialChannel;

/// What one material map writes into its image: the whole base color, the
/// emissive color, or a channel packing of one [`MaterialChannel`] per RGBA
/// channel.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialBake {
    /// The straight-RGBA base color from the `baseColor` attribute, four
    /// channels.
    RgbaColor,

    /// The `emissiveColor` color, three channels (RGB), opaque. The
    /// `emissiveStrength` rides on the material as a flat
    /// `KHR_materials_emissive_strength` factor, not folded into the texel.
    EmissiveColor,

    /// A channel packing, one to four [`MaterialChannel`]s in `R`, `G`, `B`,
    /// `A` order. An unnamed trailing channel is `0`, and alpha defaults to
    /// opaque when the packing has fewer than four channels.
    Packing(Vec<MaterialChannel>),
}
