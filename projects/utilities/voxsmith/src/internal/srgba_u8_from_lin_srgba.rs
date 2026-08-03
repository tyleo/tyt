use ty_math::{TyLinSrgbaF64, TySrgbaF64, TySrgbaU8};

/// Encodes a linear-light color to 8-bit sRGB, the display- and export-side
/// half of the one transfer voxsmith applies at its boundaries. On every
/// 8-bit code it is the exact inverse of
/// [`lin_srgba_from_srgba_u8`](crate::lin_srgba_from_srgba_u8). Each
/// color component encodes through the sRGB transfer and quantizes, clamped
/// to `[0, 1]`. The alpha carries no gamma, so it only quantizes.
pub(crate) fn srgba_u8_from_lin_srgba(color: TyLinSrgbaF64) -> TySrgbaU8 {
    TySrgbaF64::from_linear(color).into_format::<u8, u8>()
}

#[cfg(test)]
mod tests {
    use super::srgba_u8_from_lin_srgba;
    use crate::lin_srgba_from_srgba_u8;
    use ty_math::TySrgbaU8;

    /// Every 8-bit code survives decode then encode: an imported 8-bit
    /// palette round-trips to the same bytes, including through the piecewise
    /// toe of the sRGB curve near zero.
    #[test]
    fn every_u8_code_round_trips_through_linear() {
        for code in 0..=255u8 {
            let color = TySrgbaU8::new(code, code, code, code);
            let round_tripped = srgba_u8_from_lin_srgba(lin_srgba_from_srgba_u8(color));
            assert_eq!(round_tripped, color, "code {code} did not round-trip");
        }
    }

    /// The decode matches the sRGB specification's analytic transfer inverse.
    #[test]
    fn decode_matches_the_reference_transfer() {
        for code in 0..=255u8 {
            let srgb = code as f64 / 255.0;
            let linear = lin_srgba_from_srgba_u8(TySrgbaU8::new(code, 0, 0, 255)).red;
            assert!(
                (linear - srgb_to_linear(srgb)).abs() < 1e-12,
                "code {code} decoded to {linear}"
            );
        }
    }

    /// The alpha never passes through the transfer: it normalizes on decode
    /// and quantizes on encode, straight both ways.
    #[test]
    fn alpha_is_never_transfer_encoded() {
        for code in [0u8, 1, 64, 128, 191, 254, 255] {
            let alpha = lin_srgba_from_srgba_u8(TySrgbaU8::new(0, 0, 0, code)).alpha;
            assert_eq!(alpha, code as f64 / 255.0, "alpha {code} decoded curved");
        }
    }

    /// The independent forward transfer recovers each code's sRGB fraction
    /// from the production decode, so the decode inverts under a reference
    /// the production encode never touches.
    #[test]
    fn reference_forward_transfer_inverts_the_decode() {
        for code in 0..=255u8 {
            let linear = lin_srgba_from_srgba_u8(TySrgbaU8::new(code, 0, 0, 255)).red;
            let srgb = linear_to_srgb(linear);
            assert!(
                (srgb - code as f64 / 255.0).abs() < 1e-12,
                "code {code} re-encoded to {srgb}"
            );
        }
    }

    /// The sRGB transfer inverse, kept as an independent reference for the
    /// production decode (`palette`'s `into_linear`).
    fn srgb_to_linear(component: f64) -> f64 {
        if component <= 0.040_45 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }

    /// The forward sRGB transfer, kept as an independent reference to prove
    /// the production decode inverts.
    fn linear_to_srgb(linear: f64) -> f64 {
        if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        }
    }
}
