/// How a material's base color alpha is rendered.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MeshAlphaMode {
    /// The alpha is ignored and the surface is fully opaque.
    #[default]
    Opaque,

    /// The surface is fully opaque or fully transparent by the alpha against
    /// the material's cutoff.
    Mask,

    /// The alpha blends the surface over what is behind it.
    Blend,
}
