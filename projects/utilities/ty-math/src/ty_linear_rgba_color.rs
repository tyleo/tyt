use crate::{TyCielabColorF64, TyOklabColorF64, TySrgbaColor};

/// A color in linear RGB with straight alpha, generic over its component type
/// `T`. Linear light is the space perceptual conversions build on; components
/// are nominally `[0, 1]` but may exceed it out of gamut.
///
/// The component type defaults to `f32`, so `TyLinearRgbaColor` is the `f32`
/// color; see `TyLinearRgbaColorF32` and `TyLinearRgbaColorF64`. The conversions
/// are defined on the `f64` instantiation, where the perceptual math keeps full
/// precision.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyLinearRgbaColor<T = f32> {
    /// The red component.
    pub r: T,

    /// The green component.
    pub g: T,

    /// The blue component.
    pub b: T,

    /// The straight-alpha component.
    pub a: T,
}

impl<T> TyLinearRgbaColor<T> {
    /// Creates a color from its components.
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
}

impl TyLinearRgbaColor<f64> {
    /// Encodes to the 8-bit sRGB storage color: `r` / `g` / `b` through the sRGB
    /// transfer function and `a` scaled straight, each clamped to `[0, 1]` and
    /// rounded.
    pub fn to_srgba(self) -> TySrgbaColor {
        TySrgbaColor::new(
            linear_to_srgb_byte(self.r),
            linear_to_srgb_byte(self.g),
            linear_to_srgb_byte(self.b),
            straight_byte(self.a),
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

/// Encodes a linear `[0, 1]` component to an sRGB byte.
fn linear_to_srgb_byte(linear: f64) -> u8 {
    let c = linear.clamp(0.0, 1.0);

    let s = if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };

    (s * 255.0).round() as u8
}

/// Scales a straight `[0, 1]` value (an alpha, which carries no gamma) to a byte.
fn straight_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
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
