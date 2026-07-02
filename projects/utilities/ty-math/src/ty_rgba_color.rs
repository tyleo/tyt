use crate::{TyHsvaColor, TyVector3, ty_array_conversions};
use std::ops::Mul;

/// An RGBA color with component type `T`, each component in `[0, 1]`.
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

ty_array_conversions!(TyRgbaColor, 4, r, g, b, a);

impl<T: Copy> TyRgbaColor<T> {
    /// The `r`, `g`, and `b` components as a [`TyVector3`], dropping alpha.
    pub fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.r, self.g, self.b)
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for TyRgbaColor<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

/// Implements the float-only color operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_rgba_color_float {
    ($t:ty) => {
        impl TyRgbaColor<$t> {
            /// Converts to HSVA. A gray color (zero chroma) reports a hue of `0`.
            pub fn to_hsva(self) -> TyHsvaColor<$t> {
                let max = self.r.max(self.g).max(self.b);
                let min = self.r.min(self.g).min(self.b);
                let delta = max - min;

                let value = max;
                let saturation = if max <= 0.0 { 0.0 } else { delta / max };

                let hue = if delta > 0.0 {
                    // The sextant is set by whichever channel is the maximum.
                    let sextant = if self.r >= self.g && self.r >= self.b {
                        let hue = (self.g - self.b) / delta;
                        if hue < 0.0 { hue + 6.0 } else { hue }
                    } else if self.g >= self.b {
                        2.0 + (self.b - self.r) / delta
                    } else {
                        4.0 + (self.r - self.g) / delta
                    };
                    sextant / 6.0
                } else {
                    0.0
                };

                TyHsvaColor::new(hue, saturation, value, self.a)
            }
        }

        impl Mul<TyRgbaColor<$t>> for $t {
            type Output = TyRgbaColor<$t>;

            fn mul(self, rhs: TyRgbaColor<$t>) -> TyRgbaColor<$t> {
                rhs * self
            }
        }
    };
}

impl_ty_rgba_color_float!(f32);
impl_ty_rgba_color_float!(f64);
