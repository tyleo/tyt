/// A color in the OKLab perceptual space with straight alpha, generic over its
/// component type `T`: perceptual lightness `l`, the `a` green-red opponent
/// axis, and the `b` blue-yellow opponent axis.
///
/// The component type defaults to `f32`, so `TyOklabColor` is the `f32` color;
/// see `TyOklabColorF32` and `TyOklabColorF64`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyOklabColor<T = f32> {
    /// Perceptual lightness.
    pub l: T,

    /// The green-red opponent axis.
    pub a: T,

    /// The blue-yellow opponent axis.
    pub b: T,

    /// The straight-alpha component.
    pub alpha: T,
}

impl<T> TyOklabColor<T> {
    /// Creates a color from its lightness, opponent axes, and alpha.
    pub fn new(l: T, a: T, b: T, alpha: T) -> Self {
        Self { l, a, b, alpha }
    }
}
