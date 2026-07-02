use crate::{TyLinearRgbaColorF64, TyRgbaColorF64, ty_array_conversions};

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
    use crate::TySrgbaColor;

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
}
