use crate::{TyLinearRgbaColorF64, TyRgbaColorF64, TyVector3, ty_array_conversions};

/// An 8-bit sRGB color with straight alpha, the `#RRGGBBAA` storage form:
/// `r` / `g` / `b` gamma-encoded, `a` linear. Decode with
/// [`to_linear_rgba`](Self::to_linear_rgba) to compute in linear light.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TySrgbaColor {
    /// The red component.
    pub r: u8,

    /// The green component.
    pub g: u8,

    /// The blue component.
    pub b: u8,

    /// The straight-alpha component (linear, no gamma).
    pub a: u8,
}

ty_array_conversions!(TySrgbaColor, u8, 4, r, g, b, a);

impl TySrgbaColor {
    /// Creates a color from its 8-bit components.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

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

    /// Formats the color as an uppercase `#RRGGBBAA` hex string, the `#RRGGBBAA`
    /// storage form. Round-trips with [`from_hex`](Self::from_hex).
    pub fn to_hex(self) -> String {
        let Self { r, g, b, a } = self;

        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }

    /// Normalizes each 8-bit component straight to `[0, 1]`, without decoding
    /// the sRGB transfer function. This is the gamma-encoded color as floats;
    /// use [`to_linear_rgba`](Self::to_linear_rgba) to compute in linear light.
    pub fn to_rgba(self) -> TyRgbaColorF64 {
        TyRgbaColorF64::new(
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
            self.a as f64 / 255.0,
        )
    }

    /// Normalizes the `r` / `g` / `b` components straight to `[0, 1]` as a
    /// [`TyVector3`], dropping alpha and without decoding the sRGB transfer
    /// function. The 8-bit parallel to
    /// [`TyRgbaColor::to_vector3`](crate::TyRgbaColor::to_vector3).
    pub fn to_vector3(self) -> TyVector3<f64> {
        self.to_rgba().to_vector3()
    }

    /// Decodes to linear RGB: the sRGB transfer function inverted on `r` / `g` /
    /// `b`, and `a` scaled straight since alpha carries no gamma.
    pub fn to_linear_rgba(self) -> TyLinearRgbaColorF64 {
        TyLinearRgbaColorF64::new(
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            self.a as f64 / 255.0,
        )
    }
}

/// Inverts the sRGB transfer function for one 8-bit component to linear `[0, 1]`.
fn srgb_to_linear(byte: u8) -> f64 {
    let c = byte as f64 / 255.0;

    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TySrgbaColor, TyVector3F64};

    #[test]
    fn to_vector3_normalizes_and_drops_alpha() {
        let rgb = TySrgbaColor::new(255, 128, 0, 64).to_vector3();
        assert_eq!(rgb, TyVector3F64::new(1.0, 128.0 / 255.0, 0.0));
    }

    #[test]
    fn array_converts_both_ways() {
        // The concrete component type gets the reverse `From` a generic one
        // cannot.
        let color = TySrgbaColor::from_array([1, 2, 3, 4]);
        assert_eq!(color, TySrgbaColor::new(1, 2, 3, 4));

        let array: [u8; 4] = color.into();
        assert_eq!(array, [1, 2, 3, 4]);

        let from_array: TySrgbaColor = [10, 20, 30, 40].into();
        assert_eq!(from_array, TySrgbaColor::new(10, 20, 30, 40));
    }

    #[test]
    fn from_slice_reads_the_first_components() {
        assert_eq!(
            TySrgbaColor::from_slice(&[5, 6, 7, 8, 9]),
            TySrgbaColor::new(5, 6, 7, 8)
        );
    }

    #[test]
    fn hex_round_trips_and_defaults_alpha() {
        // Eight digits carry alpha; six default it to opaque; the `#` is
        // optional either way.
        assert_eq!(
            TySrgbaColor::from_hex("#01020304"),
            Some(TySrgbaColor::new(1, 2, 3, 4))
        );
        assert_eq!(
            TySrgbaColor::from_hex("FF8000"),
            Some(TySrgbaColor::new(255, 128, 0, 255))
        );
        assert_eq!(TySrgbaColor::new(1, 2, 3, 4).to_hex(), "#01020304");
        assert_eq!(
            TySrgbaColor::from_hex(&TySrgbaColor::new(10, 200, 30, 40).to_hex()),
            Some(TySrgbaColor::new(10, 200, 30, 40))
        );
    }

    #[test]
    fn from_hex_rejects_malformed() {
        // Wrong length and non-hex digits are both rejected.
        assert_eq!(TySrgbaColor::from_hex("#12345"), None);
        assert_eq!(TySrgbaColor::from_hex("#GGGGGG"), None);
        assert_eq!(TySrgbaColor::from_hex(""), None);
    }
}
