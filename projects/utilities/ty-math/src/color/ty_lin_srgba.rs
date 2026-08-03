use palette::LinSrgba;

/// A linear-light RGBA color with straight alpha, backed by
/// [`palette::LinSrgba`], the linear counterpart of [`TySrgba`](crate::TySrgba).
/// Components are nominally `[0, 1]` and may exceed it out of gamut.
pub type TyLinSrgba<T = f32> = LinSrgba<T>;

#[cfg(test)]
mod tests {
    use crate::{TyLinSrgbaF64, TySrgbaF64};

    #[test]
    fn componentwise_multiply_tints_each_channel() {
        // By-value `*` is the Hadamard product, alpha included, replacing tyt's
        // `componentwise_multiply`.
        let tinted =
            TyLinSrgbaF64::new(0.5, 0.8, 1.0, 1.0) * TyLinSrgbaF64::new(0.5, 0.0, 0.25, 0.5);
        assert_eq!(tinted, TyLinSrgbaF64::new(0.25, 0.0, 0.25, 0.5));
    }

    #[test]
    fn into_linear_decodes_and_round_trips() {
        // The sRGB endpoints decode exactly and alpha passes straight; a mid
        // channel survives decode -> re-encode within floating-point tolerance.
        let decoded = TySrgbaF64::new(0.0, 1.0, 0.5, 0.25).into_linear();
        assert_eq!(
            (decoded.red, decoded.green, decoded.alpha),
            (0.0, 1.0, 0.25)
        );

        let round = TySrgbaF64::from_linear(decoded);
        assert!((round.red - 0.0).abs() < 1e-9);
        assert!((round.green - 1.0).abs() < 1e-9);
        assert!((round.blue - 0.5).abs() < 1e-9);
        assert_eq!(round.alpha, 0.25);
    }
}
