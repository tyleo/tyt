/// A texture's minification filter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MeshMinFilter {
    /// The nearest texel.
    Nearest,

    /// A linear blend of the nearest texels.
    Linear,

    /// The nearest texel of the nearest mipmap.
    NearestMipmapNearest,

    /// A linear blend within the nearest mipmap.
    LinearMipmapNearest,

    /// The nearest texel, blended between the two nearest mipmaps.
    NearestMipmapLinear,

    /// A linear blend within and between the two nearest mipmaps.
    LinearMipmapLinear,
}
