/// A color in the CIELAB perceptual space with straight alpha, generic over its
/// component type `T`: lightness `l`, the `a` green-red axis, and the `b`
/// blue-yellow axis, under the D65 white point.
///
/// The component type defaults to `f32`, so `TyCielabColor` is the `f32` color;
/// see `TyCielabColorF32` and `TyCielabColorF64`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyCielabColor<T = f32> {
    /// Lightness.
    pub l: T,

    /// The green-red axis.
    pub a: T,

    /// The blue-yellow axis.
    pub b: T,

    /// The straight-alpha component.
    pub alpha: T,
}

impl<T> TyCielabColor<T> {
    /// Creates a color from its lightness, axes, and alpha.
    pub fn new(l: T, a: T, b: T, alpha: T) -> Self {
        Self { l, a, b, alpha }
    }
}
