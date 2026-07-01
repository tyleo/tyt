use crate::TyLinearRgbaColorF64;

/// A color in the sRGB color space with straight alpha, stored as the 8-bit
/// `#RRGGBBAA` code: `r` / `g` / `b` gamma-encoded, `a` linear. This is the
/// storage form; decode to [`TyLinearRgbaColorF64`] to compute in linear light.
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

impl TySrgbaColor {
    /// Creates a color from its 8-bit components.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
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
