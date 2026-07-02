use crate::{TyVector3, ty_array_conversions};

/// A CIELAB perceptual color with straight alpha and component type `T`, under
/// the D65 white point.
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

ty_array_conversions!(TyCielabColor, 4, l, a, b, alpha);

impl<T: Copy> TyCielabColor<T> {
    /// The `l`, `a`, and `b` axes as a [`TyVector3`], dropping alpha.
    pub fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.l, self.a, self.b)
    }
}
