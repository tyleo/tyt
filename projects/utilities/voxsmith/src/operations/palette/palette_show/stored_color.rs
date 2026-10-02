use treegrid::TreeGridJsonValue;
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbF64, TySrgbaF64};

/// A stored linear color of three or four components under the color
/// readings. Each width has its own impl, so a reading matches the width once
/// per value pool.
pub trait StoredColor {
    fn components(&self) -> &[f64];

    /// The `[r, g, b, a]` bytes encoded to sRGB for display. The alpha only
    /// quantizes. A three-component color takes opaque alpha.
    fn display_bytes(&self) -> [u8; 4];

    /// The whole color under `linear-float`: the stored linear color.
    fn linear_float(&self) -> TreeGridJsonValue;

    /// The whole color under `srgb-float`: the rgb components
    /// transfer-encoded, alpha passed through.
    fn srgb_float(&self) -> TreeGridJsonValue;

    /// The whole color under `srgb-hex`: its sRGB bytes.
    fn srgb_hex(&self) -> TreeGridJsonValue;

    /// Rgb component `index` under `srgb-float`: transfer-encoded.
    fn srgb_float_rgb_component(&self, index: usize) -> f64 {
        srgb_float_encoded(self.components()[index])
    }
}

impl StoredColor for [f64; 3] {
    fn components(&self) -> &[f64] {
        self
    }

    fn display_bytes(&self) -> [u8; 4] {
        let [red, green, blue] = *self;
        srgba_bytes(TyLinSrgbaF64::new(red, green, blue, 1.0))
    }

    fn linear_float(&self) -> TreeGridJsonValue {
        let [red, green, blue] = *self;
        TreeGridJsonValue::lin_srgb(TyLinSrgbF64::new(red, green, blue))
    }

    fn srgb_float(&self) -> TreeGridJsonValue {
        let [red, green, blue] = self.map(srgb_float_encoded);
        TreeGridJsonValue::srgb(TySrgbF64::new(red, green, blue))
    }

    fn srgb_hex(&self) -> TreeGridJsonValue {
        let [red, green, blue, _] = self.display_bytes();
        TreeGridJsonValue::srgb8([red, green, blue])
    }
}

impl StoredColor for [f64; 4] {
    fn components(&self) -> &[f64] {
        self
    }

    fn display_bytes(&self) -> [u8; 4] {
        let [red, green, blue, alpha] = *self;
        srgba_bytes(TyLinSrgbaF64::new(red, green, blue, alpha))
    }

    fn linear_float(&self) -> TreeGridJsonValue {
        let [red, green, blue, alpha] = *self;
        TreeGridJsonValue::lin_srgba(TyLinSrgbaF64::new(red, green, blue, alpha))
    }

    fn srgb_float(&self) -> TreeGridJsonValue {
        let [red, green, blue, alpha] = *self;
        TreeGridJsonValue::srgba(TySrgbaF64::new(
            srgb_float_encoded(red),
            srgb_float_encoded(green),
            srgb_float_encoded(blue),
            alpha,
        ))
    }

    fn srgb_hex(&self) -> TreeGridJsonValue {
        TreeGridJsonValue::srgba8(self.display_bytes())
    }
}

/// The sRGB bytes for the linear `color`. The alpha only quantizes.
fn srgba_bytes(color: TyLinSrgbaF64) -> [u8; 4] {
    <[u8; 4]>::from(TySrgbaF64::from_linear(color).into_format::<u8, u8>())
}

/// A linear rgb component transfer-encoded and rounded to the six decimal
/// places `srgb-float` displays.
fn srgb_float_encoded(component: f64) -> f64 {
    let encoded = TySrgbF64::from_linear(TyLinSrgbF64::new(component, 0.0, 0.0)).red;
    (encoded * 1e6).round() / 1e6
}
