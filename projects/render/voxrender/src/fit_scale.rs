use crate::fit_radius;
use ty_math::TyBoundsF64;

/// The world units across the shorter image axis at which an orthographic
/// view fits the bounding sphere of `bounds` with
/// [`FIT_MARGIN`](crate::FIT_MARGIN) around it.
pub fn fit_scale(bounds: &TyBoundsF64) -> f64 {
    2.0 * fit_radius(bounds)
}

#[cfg(test)]
mod tests {
    use crate::{FIT_MARGIN, fit_scale};
    use ty_math::{TyBoundsF64, TyVector3F64};

    #[test]
    fn the_scale_spans_the_sphere() {
        let bounds = TyBoundsF64::new(TyVector3F64::ONE, TyVector3F64::new(3.0, 4.0, 0.0));

        assert!((fit_scale(&bounds) - 10.0 * (1.0 + FIT_MARGIN)).abs() < 1e-9);
    }
}
