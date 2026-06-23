/// An RGBA color generic over its component type `T`, each component in the
/// range `[0, 1]`.
///
/// The component type defaults to `f32`, so `TyRgbaColor` is the `f32` color;
/// see `TyRgbaColorF32` and `TyRgbaColorF64` for the common instantiations.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyRgbaColor<T = f32> {
    /// The red component, in the range `[0, 1]`.
    pub r: T,

    /// The green component, in the range `[0, 1]`.
    pub g: T,

    /// The blue component, in the range `[0, 1]`.
    pub b: T,

    /// The alpha component, in the range `[0, 1]`.
    pub a: T,
}

impl<T> TyRgbaColor<T> {
    /// Creates a new color from `r`, `g`, `b`, and `a` components.
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
}
