use crate::utilities::QuantizePoint;
use ty_math::TyVector4F64;

/// The `(min, max)` corners of a point set's coordinates, per axis.
pub fn point_bounds(points: &[QuantizePoint]) -> (TyVector4F64, TyVector4F64) {
    let mut low = TyVector4F64::INFINITY;
    let mut high = TyVector4F64::NEG_INFINITY;

    for point in points {
        low = low.min(point.coords);
        high = high.max(point.coords);
    }

    (low, high)
}
