/// A texture's magnification filter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MeshMagFilter {
    /// The nearest texel.
    Nearest,

    /// A linear blend of the nearest texels.
    Linear,
}
