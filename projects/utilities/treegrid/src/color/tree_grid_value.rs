use crate::{TreeGridSwatch, TreeGridValue};
use std::fmt::Display;
use ty_math::{
    TyFloatExt, TyLinSrgb, TyLinSrgbF64, TyLinSrgba, TyLinSrgbaF64, TySrgb, TySrgbF64, TySrgba,
    TySrgbaF64,
};

// The `TyFloatExt` bound pins `T` to f32 / f64: the 8-bit forms go
// through `srgb8` / `srgba8`, not the float quantize. Color math
// widens losslessly to f64, where ty-math defines the conversions;
// text formats the `T` components directly, so an f32 keeps its
// shortest `Display` form.
impl TreeGridValue {
    /// Creates a float-component sRGB color value: `srgb(r, g, b)`
    /// functional text, with a color swatch quantizing each
    /// component to a byte.
    pub fn srgb<T: Copy + Display + TyFloatExt + Into<f64>>(color: TySrgb<T>) -> Self {
        let [r, g, b] = <[T; 3]>::from(color);
        let bytes = TySrgbF64::new(r.into(), g.into(), b.into()).into_format::<u8>();
        Self::new(format!("srgb({r}, {g}, {b})")).with_swatch(TreeGridSwatch::Color(bytes.into()))
    }

    /// Creates a float-component sRGB color value with alpha:
    /// `srgba(r, g, b, a)` functional text, with a color swatch of
    /// the quantized rgb part.
    pub fn srgba<T: Copy + Display + TyFloatExt + Into<f64>>(color: TySrgba<T>) -> Self {
        let [r, g, b, a] = <[T; 4]>::from(color);
        let bytes = TySrgbaF64::new(r.into(), g.into(), b.into(), a.into())
            .into_format::<u8, u8>()
            .color;
        Self::new(format!("srgba({r}, {g}, {b}, {a})"))
            .with_swatch(TreeGridSwatch::Color(bytes.into()))
    }

    /// Creates a float-component linear RGB color value:
    /// `lin_srgb(r, g, b)` functional text, with a color swatch
    /// transfer-encoding each component to sRGB and quantizing it to
    /// a byte. Out-of-gamut components clamp in the quantize.
    pub fn lin_srgb<T: Copy + Display + TyFloatExt + Into<f64>>(color: TyLinSrgb<T>) -> Self {
        let [r, g, b] = <[T; 3]>::from(color);
        let linear = TyLinSrgbF64::new(r.into(), g.into(), b.into());
        let bytes = TySrgbF64::from_linear(linear).into_format::<u8>();
        Self::new(format!("lin_srgb({r}, {g}, {b})"))
            .with_swatch(TreeGridSwatch::Color(bytes.into()))
    }

    /// Creates a float-component linear RGB color value with alpha:
    /// `lin_srgba(r, g, b, a)` functional text, with a color swatch
    /// of the transfer-encoded, quantized rgb part. Out-of-gamut
    /// components clamp in the quantize.
    pub fn lin_srgba<T: Copy + Display + TyFloatExt + Into<f64>>(color: TyLinSrgba<T>) -> Self {
        let [r, g, b, a] = <[T; 4]>::from(color);
        let linear = TyLinSrgbaF64::new(r.into(), g.into(), b.into(), a.into());
        let bytes = TySrgbaF64::from_linear(linear)
            .into_format::<u8, u8>()
            .color;
        Self::new(format!("lin_srgba({r}, {g}, {b}, {a})"))
            .with_swatch(TreeGridSwatch::Color(bytes.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridSwatch, TreeGridValue};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbF32, TySrgbF64, TySrgbaF64};

    #[test]
    fn srgb_renders_functional_text_with_a_quantized_swatch() {
        let value = TreeGridValue::srgb(TySrgbF64::new(1.0, 0.0, 0.5));

        assert_eq!(value.text, "srgb(1, 0, 0.5)");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([255, 0, 128])));
    }

    #[test]
    fn srgba_includes_alpha_in_text_and_drops_it_from_the_swatch() {
        let value = TreeGridValue::srgba(TySrgbaF64::new(1.0, 0.0, 0.5, 0.25));

        assert_eq!(value.text, "srgba(1, 0, 0.5, 0.25)");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([255, 0, 128])));
    }

    #[test]
    fn lin_srgb_transfer_encodes_the_swatch() {
        let value = TreeGridValue::lin_srgb(TyLinSrgbF64::new(0.0, 1.0, 0.5));

        assert_eq!(value.text, "lin_srgb(0, 1, 0.5)");
        // Linear 0.5 encodes to sRGB byte 188 through the transfer.
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([0, 255, 188])));
    }

    #[test]
    fn lin_srgba_clamps_an_hdr_component_in_the_swatch() {
        let value = TreeGridValue::lin_srgba(TyLinSrgbaF64::new(2.0, 1.0, 0.5, 1.0));

        // The HDR red keeps full fidelity in the text and saturates
        // the swatch byte.
        assert_eq!(value.text, "lin_srgba(2, 1, 0.5, 1)");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([255, 255, 188])));
    }

    #[test]
    fn f32_components_render_their_own_display_form() {
        let value = TreeGridValue::srgb(TySrgbF32::new(0.25, 0.5, 1.0));

        assert_eq!(value.text, "srgb(0.25, 0.5, 1)");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([64, 128, 255])));
    }
}
