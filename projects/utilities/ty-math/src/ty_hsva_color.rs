use crate::{TyFloatExt, TyRgbaColor, ty_array_conversions};

/// An HSVA color with component type `T`, each component in `[0, 1]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyHsvaColor<T = f32> {
    /// The hue, in `[0, 1]`.
    pub h: T,

    /// The saturation, in `[0, 1]`.
    pub s: T,

    /// The value, in `[0, 1]`.
    pub v: T,

    /// The straight-alpha component, in `[0, 1]`.
    pub a: T,
}

impl<T> TyHsvaColor<T> {
    /// Creates a color from its hue, saturation, value, and alpha.
    pub fn new(h: T, s: T, v: T, a: T) -> Self {
        Self { h, s, v, a }
    }
}

ty_array_conversions!(TyHsvaColor, 4, h, s, v, a);

/// Implements the float-only color operations for a concrete floating-point
/// component type.
macro_rules! impl_ty_hsva_color_float {
    ($t:ty) => {
        impl TyHsvaColor<$t> {
            /// Converts to RGBA. The hue wraps into `[0, 1)` first, and alpha
            /// passes through unchanged.
            pub fn to_rgba(self) -> TyRgbaColor<$t> {
                if self.s <= 0.0 {
                    return TyRgbaColor::new(self.v, self.v, self.v, self.a);
                }

                let hue = self.h.wrap01() * 6.0;
                let sextant = hue as u32;
                let fraction = hue - sextant as $t;

                let p = self.v * (1.0 - self.s);
                let q = self.v * (1.0 - self.s * fraction);
                let t = self.v * (1.0 - self.s * (1.0 - fraction));

                let (r, g, b) = match sextant % 6 {
                    0 => (self.v, t, p),
                    1 => (q, self.v, p),
                    2 => (p, self.v, t),
                    3 => (p, q, self.v),
                    4 => (t, p, self.v),
                    _ => (self.v, p, q),
                };

                TyRgbaColor::new(r, g, b, self.a)
            }

            /// Linearly interpolates from `self` towards `other` by `t`, taking
            /// the shortest arc around the hue circle.
            pub fn lerp(self, other: Self, t: $t) -> Self {
                let mut delta_hue = other.h - self.h;

                // Wrap into [-0.5, 0.5] to cross the shortest way round the wheel.
                if delta_hue > 0.5 {
                    delta_hue -= 1.0;
                } else if delta_hue < -0.5 {
                    delta_hue += 1.0;
                }

                Self::new(
                    (self.h + delta_hue * t).wrap01(),
                    self.s + (other.s - self.s) * t,
                    self.v + (other.v - self.v) * t,
                    self.a + (other.a - self.a) * t,
                )
            }
        }
    };
}

impl_ty_hsva_color_float!(f32);
impl_ty_hsva_color_float!(f64);

#[cfg(test)]
mod tests {
    use crate::{TyHsvaColorF64, TyRgbaColorF64};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_rgba(a: TyRgbaColorF64, b: TyRgbaColorF64) -> bool {
        close(a.r, b.r) && close(a.g, b.g) && close(a.b, b.b) && close(a.a, b.a)
    }

    #[test]
    fn primary_hues_convert_to_rgb() {
        // Hue 0 is red, 1/3 is green, 2/3 is blue, all fully saturated.
        assert!(close_rgba(
            TyHsvaColorF64::new(0.0, 1.0, 1.0, 1.0).to_rgba(),
            TyRgbaColorF64::new(1.0, 0.0, 0.0, 1.0)
        ));
        assert!(close_rgba(
            TyHsvaColorF64::new(1.0 / 3.0, 1.0, 1.0, 1.0).to_rgba(),
            TyRgbaColorF64::new(0.0, 1.0, 0.0, 1.0)
        ));
        assert!(close_rgba(
            TyHsvaColorF64::new(2.0 / 3.0, 1.0, 1.0, 1.0).to_rgba(),
            TyRgbaColorF64::new(0.0, 0.0, 1.0, 1.0)
        ));
    }

    #[test]
    fn to_rgba_keeps_alpha_when_desaturated() {
        // A zero-saturation color is gray at its value, with its alpha intact.
        let rgba = TyHsvaColorF64::new(0.4, 0.0, 0.5, 0.25).to_rgba();
        assert!(close_rgba(rgba, TyRgbaColorF64::new(0.5, 0.5, 0.5, 0.25)));
    }

    #[test]
    fn round_trips_through_rgba() {
        let hsva = TyHsvaColorF64::new(0.6, 0.7, 0.8, 0.9);
        let back = hsva.to_rgba().to_hsva();
        assert!(close(back.h, hsva.h) && close(back.s, hsva.s));
        assert!(close(back.v, hsva.v) && close(back.a, hsva.a));
    }

    #[test]
    fn lerp_takes_the_short_way_round_the_wheel() {
        // From hue 0.9 to 0.1 the short arc passes through 1.0/0.0, not back
        // through 0.5, so the midpoint is hue 0.
        let mid = TyHsvaColorF64::new(0.9, 1.0, 1.0, 1.0)
            .lerp(TyHsvaColorF64::new(0.1, 1.0, 1.0, 1.0), 0.5);
        assert!(close(mid.h, 0.0));
    }
}
