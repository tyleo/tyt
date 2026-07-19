use crate::{TySrgb, linear_to_srgb, ty_array_conversions};

/// A linear-light RGB color without alpha, the three-component companion to
/// [`TyLinSrgba`](crate::TyLinSrgba). Components are nominally `[0, 1]` and may
/// exceed it out of gamut. Conversions are defined on the `f64` instantiation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TyLinSrgb<T = f32> {
    /// The red component.
    pub r: T,

    /// The green component.
    pub g: T,

    /// The blue component.
    pub b: T,
}

impl<T> TyLinSrgb<T> {
    /// Creates a color from its components.
    pub fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

ty_array_conversions!(TyLinSrgb, 3, r, g, b);

impl TyLinSrgb<f64> {
    /// Encodes to sRGB: the sRGB transfer function on each component.
    /// Out-of-gamut components extend by sign rather than clamp; quantize to
    /// the 8-bit form with [`TySrgb::to_u8`].
    pub fn to_srgb(self) -> TySrgb<f64> {
        TySrgb::new(
            linear_to_srgb(self.r),
            linear_to_srgb(self.g),
            linear_to_srgb(self.b),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyLinSrgbF64, TySrgbU8};

    #[test]
    fn array_round_trips() {
        let color = TyLinSrgbF64::from_array([0.25, 0.5, 0.75]);
        assert_eq!(color, TyLinSrgbF64::new(0.25, 0.5, 0.75));
        assert_eq!(color.to_array(), [0.25, 0.5, 0.75]);
    }

    #[test]
    fn to_srgb_then_u8_encodes_and_clamps() {
        // The transfer maps the endpoints exactly and the byte quantize clamps
        // out-of-gamut components, matching the four-component type.
        assert_eq!(
            TyLinSrgbF64::new(0.0, 1.0, 2.0).to_srgb().to_u8(),
            TySrgbU8::new(0, 255, 255)
        );
    }
}
