use crate::{TyVector3, ty_array_conversions};

/// An OKLab perceptual color with straight alpha and component type `T`.
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

ty_array_conversions!(TyOklabColor, 4, l, a, b, alpha);

impl<T: Copy> TyOklabColor<T> {
    /// The `l`, `a`, and `b` axes as a [`TyVector3`], dropping alpha.
    pub fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.l, self.a, self.b)
    }
}
