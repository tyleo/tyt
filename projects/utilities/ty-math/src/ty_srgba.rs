use crate::{TyFloatExt, TyLinSrgba, TySrgb, ty_array_conversions};
use std::{
    hash::{Hash, Hasher},
    ops::Mul,
};

/// An sRGB color with straight alpha. The color space is the type identity;
/// `T` is the orthogonal storage axis: `u8` is the `#RRGGBBAA` byte form,
/// `f32` / `f64` are normalized `[0, 1]`. Decode to linear before lighting math.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TySrgba<T = f32> {
    /// The red component, gamma-encoded.
    pub r: T,

    /// The green component, gamma-encoded.
    pub g: T,

    /// The blue component, gamma-encoded.
    pub b: T,

    /// The straight-alpha component.
    pub a: T,
}

impl<T> TySrgba<T> {
    /// Creates a color from its components.
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }
}

ty_array_conversions!(TySrgba, 4, r, g, b, a);

impl<T: Copy> TySrgba<T> {
    /// The `r`, `g`, and `b` components as a [`TySrgb`], dropping alpha.
    pub fn to_srgb(&self) -> TySrgb<T> {
        TySrgb::new(self.r, self.g, self.b)
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for TySrgba<T> {
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

// `Eq` / `Hash` hold only for the integer storage, so the 8-bit color can key a
// dedup map (the `MaterialKey` palette key); `f32` / `f64` get `PartialEq` from
// the derive alone. The manual `Hash` mirrors the derived field-order hashing.
impl Eq for TySrgba<u8> {}

impl Hash for TySrgba<u8> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.hash(state);
        self.g.hash(state);
        self.b.hash(state);
        self.a.hash(state);
    }
}

impl TySrgba<u8> {
    /// Parses a `#RRGGBB` or `#RRGGBBAA` hex string, with or without the leading
    /// `#`. A missing alpha defaults to opaque. Returns `None` when the value is
    /// not six or eight hexadecimal digits.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let byte = |index: usize| u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok();

        Some(Self {
            r: byte(0)?,
            g: byte(1)?,
            b: byte(2)?,
            a: if hex.len() == 8 { byte(3)? } else { 255 },
        })
    }

    /// Formats the color as an uppercase `#RRGGBBAA` hex string. Round-trips
    /// with [`from_hex`](Self::from_hex).
    pub fn to_hex(self) -> String {
        let Self { r, g, b, a } = self;

        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }

    /// Normalizes each 8-bit component straight to `[0, 1]` without the sRGB
    /// transfer function; the gamma-encoded color as floats. The inverse of
    /// `to_u8`. Decode to linear light with `to_f64` then `to_lin_srgba`.
    pub fn to_f64(self) -> TySrgba<f64> {
        TySrgba::new(
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
            self.a as f64 / 255.0,
        )
    }
}

impl TySrgba<f64> {
    /// Quantizes each `[0, 1]` component to a byte without the sRGB transfer
    /// function, since the components are already sRGB-encoded. The inverse of
    /// `to_f64`. Encode from linear light on the linear type instead.
    pub fn to_u8(self) -> TySrgba<u8> {
        TySrgba::new(
            self.r.to_unorm8(),
            self.g.to_unorm8(),
            self.b.to_unorm8(),
            self.a.to_unorm8(),
        )
    }

    /// Decodes to linear light: the sRGB transfer function inverted on `r` / `g`
    /// / `b`, with `a` passed straight. The inverse of [`TyLinSrgba::to_srgba`].
    pub fn to_lin_srgba(self) -> TyLinSrgba<f64> {
        TyLinSrgba::new(
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            self.a,
        )
    }
}

/// Inverts the sRGB transfer function to linear light, odd-extended by sign so
/// out-of-gamut inputs decode as `sign(x) * g(|x|)`, per CSS Color 4.
fn srgb_to_linear(c: f64) -> f64 {
    let magnitude = c.abs();

    let linear = if magnitude <= 0.040_45 {
        magnitude / 12.92
    } else {
        ((magnitude + 0.055) / 1.055).powf(2.4)
    };

    linear.copysign(c)
}

#[cfg(test)]
mod tests {
    use crate::{TySrgbaF64, TySrgbaU8};
    use std::collections::HashSet;

    #[test]
    fn hex_round_trips_and_defaults_alpha() {
        // Eight digits carry alpha; six default it to opaque; the `#` is
        // optional either way.
        assert_eq!(
            TySrgbaU8::from_hex("#01020304"),
            Some(TySrgbaU8::new(1, 2, 3, 4))
        );
        assert_eq!(
            TySrgbaU8::from_hex("FF8000"),
            Some(TySrgbaU8::new(255, 128, 0, 255))
        );
        assert_eq!(TySrgbaU8::new(1, 2, 3, 4).to_hex(), "#01020304");
        assert_eq!(
            TySrgbaU8::from_hex(&TySrgbaU8::new(10, 200, 30, 40).to_hex()),
            Some(TySrgbaU8::new(10, 200, 30, 40))
        );
    }

    #[test]
    fn from_hex_rejects_malformed() {
        // Wrong length and non-hex digits are both rejected.
        assert_eq!(TySrgbaU8::from_hex("#12345"), None);
        assert_eq!(TySrgbaU8::from_hex("#GGGGGG"), None);
        assert_eq!(TySrgbaU8::from_hex(""), None);
    }

    #[test]
    fn array_round_trips() {
        // The generic array conversions read and write component order.
        let color = TySrgbaU8::from_array([1, 2, 3, 4]);
        assert_eq!(color, TySrgbaU8::new(1, 2, 3, 4));
        assert_eq!(color.to_array(), [1, 2, 3, 4]);

        let from: TySrgbaU8 = [10, 20, 30, 40].into();
        assert_eq!(from, TySrgbaU8::new(10, 20, 30, 40));
    }

    #[test]
    fn from_slice_reads_the_first_components() {
        assert_eq!(
            TySrgbaU8::from_slice(&[5, 6, 7, 8, 9]),
            TySrgbaU8::new(5, 6, 7, 8)
        );
    }

    #[test]
    fn scalar_multiply_scales_each_component() {
        // `Mul<T>` scales every channel, alpha included; the operands are exact
        // binary fractions so the equality is precise.
        assert_eq!(
            TySrgbaF64::new(0.25, 0.5, 0.125, 1.0) * 2.0,
            TySrgbaF64::new(0.5, 1.0, 0.25, 2.0)
        );
    }

    #[test]
    fn u8_to_f64_normalizes() {
        // Straight normalize, no transfer function.
        assert_eq!(
            TySrgbaU8::new(255, 128, 0, 64).to_f64(),
            TySrgbaF64::new(1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0)
        );
    }

    #[test]
    fn f64_to_u8_quantizes_and_clamps() {
        // Out-of-range components clamp to the byte endpoints; a half rounds up.
        assert_eq!(
            TySrgbaF64::new(-0.5, 2.0, 0.5, 1.0).to_u8(),
            TySrgbaU8::new(0, 255, 128, 255)
        );
    }

    #[test]
    fn u8_round_trips_through_f64() {
        // byte -> float -> byte is exact for byte-valued components.
        let bytes = TySrgbaU8::new(0, 128, 255, 64);
        assert_eq!(bytes.to_f64().to_u8(), bytes);
    }

    #[test]
    fn to_lin_srgba_decodes_and_round_trips() {
        // The endpoints decode exactly and alpha passes straight; a mid channel
        // survives decode -> re-encode within floating-point tolerance.
        let decoded = TySrgbaF64::new(0.0, 1.0, 0.5, 0.25).to_lin_srgba();
        assert_eq!((decoded.r, decoded.g, decoded.a), (0.0, 1.0, 0.25));

        let round = decoded.to_srgba();
        assert!((round.r - 0.0).abs() < 1e-9);
        assert!((round.g - 1.0).abs() < 1e-9);
        assert!((round.b - 0.5).abs() < 1e-9);
        assert_eq!(round.a, 0.25);
    }

    #[test]
    fn u8_color_keys_a_hash_set() {
        // `Eq` + `Hash` on the 8-bit storage let it key a dedup container.
        let mut set = HashSet::new();
        set.insert(TySrgbaU8::new(1, 2, 3, 4));
        set.insert(TySrgbaU8::new(1, 2, 3, 4));
        set.insert(TySrgbaU8::new(9, 9, 9, 9));
        assert_eq!(set.len(), 2);
    }
}
