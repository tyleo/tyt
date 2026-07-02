use crate::MaterialChannel;

/// What one material map writes into its image: the whole base color, or a
/// channel packing of one [`MaterialChannel`] per RGBA channel.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialBake {
    /// The straight-RGBA base color from the `rgba` attribute, four channels.
    RgbaColor,

    /// A channel packing, one to four [`MaterialChannel`]s in `R`, `G`, `B`,
    /// `A` order. An unnamed trailing channel is `0`, and alpha defaults to
    /// opaque when the packing has fewer than four channels.
    Packing(Vec<MaterialChannel>),
}
