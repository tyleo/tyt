use crate::{TyCielabColorF64, TyOklabColorF64, TySrgba, linear_to_srgb, ty_array_conversions};
use std::ops::Mul;

/// A linear-light RGBA color with straight alpha and component type `T`, the
/// linear counterpart of [`TySrgba`]. Components are nominally `[0, 1]` and may
/// exceed it out of gamut. Conversions are defined on the `f64` instantiation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyLinSrgba<T = f32> {
    /// The red component.
    pub r: T,

    /// The green component.
    pub g: T,

    /// The blue component.
    pub b: T,

    /// The straight-alpha component.
    pub a: T,
}

impl<T> TyLinSrgba<T> {
    /// Creates a color from its components.
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
}

impl<T: Mul<Output = T> + Copy> TyLinSrgba<T> {
    /// Returns the component-wise (Hadamard) product of `self` and `other`, the
    /// tint of one color by another (a base color by its factor, say).
    pub fn componentwise_multiply(&self, other: &Self) -> Self {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }
}

ty_array_conversions!(TyLinSrgba, 4, r, g, b, a);

impl TyLinSrgba<f64> {
    /// Encodes to sRGB: the sRGB transfer function on `r` / `g` / `b`, with `a`
    /// passed straight. Out-of-gamut components extend by sign rather than
    /// clamp; quantize to the 8-bit form with `to_u8`.
    pub fn to_srgba(self) -> TySrgba<f64> {
        TySrgba::new(
            linear_to_srgb(self.r),
            linear_to_srgb(self.g),
            linear_to_srgb(self.b),
            self.a,
        )
    }

    /// Converts to OKLab.
    pub fn to_oklab(self) -> TyOklabColorF64 {
        let (r, g, b) = (self.r, self.g, self.b);

        let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
        let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
        let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;

        let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());

        TyOklabColorF64::new(
            0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
            1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
            0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
            self.a,
        )
    }

    /// Converts to CIELAB under the D65 white point.
    pub fn to_cielab(self) -> TyCielabColorF64 {
        let (r, g, b) = (self.r, self.g, self.b);

        let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
        let z = 0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b;

        let (fx, fy, fz) = (lab_f(x / 0.950_489), lab_f(y), lab_f(z / 1.088_840));

        TyCielabColorF64::new(
            116.0 * fy - 16.0,
            500.0 * (fx - fy),
            200.0 * (fy - fz),
            self.a,
        )
    }
}

/// The CIELAB nonlinearity.
fn lab_f(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;

    if t > DELTA * DELTA * DELTA {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyLinSrgbaF64, TySrgbaU8};

    #[test]
    fn componentwise_multiply_tints_each_channel() {
        let tinted = TyLinSrgbaF64::new(0.5, 0.8, 1.0, 1.0)
            .componentwise_multiply(&TyLinSrgbaF64::new(0.5, 0.0, 0.25, 0.5));
        assert_eq!(tinted, TyLinSrgbaF64::new(0.25, 0.0, 0.25, 0.5));
    }

    #[test]
    fn to_srgba_then_u8_encodes_and_clamps() {
        // The transfer maps the endpoints exactly and the byte quantize clamps
        // out-of-gamut components.
        assert_eq!(
            TyLinSrgbaF64::new(0.0, 1.0, 2.0, 1.0).to_srgba().to_u8(),
            TySrgbaU8::new(0, 255, 255, 255)
        );
    }

    #[test]
    fn to_srgba_sign_extends_out_of_gamut() {
        // Out-of-gamut components odd-extend by sign, so encode then decode
        // stays a true inverse past the [0, 1] gamut.
        let round = TyLinSrgbaF64::new(-0.5, 1.5, 0.25, 1.0)
            .to_srgba()
            .to_lin_srgba();
        assert!((round.r - (-0.5)).abs() < 1e-9);
        assert!((round.g - 1.5).abs() < 1e-9);
        assert!((round.b - 0.25).abs() < 1e-9);
        assert_eq!(round.a, 1.0);
    }
}
