use crate::FIT_MARGIN;
use ty_math::TyBoundsF64;

/// The radius of the bounding sphere of `bounds` with [`FIT_MARGIN`] around
/// it.
pub fn fit_radius(bounds: &TyBoundsF64) -> f64 {
    bounds.extents.length() * (1.0 + FIT_MARGIN)
}
