use crate::operations::sdf_doc::segment_distance2d;
use ty_math::TyVector2F64;

/// The signed distance from `point` to the simple polygon through `points`,
/// negative where the outline winds around the point.
pub fn polygon_reference(points: &[TyVector2F64], point: TyVector2F64) -> f64 {
    let edges = || {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(a, b)| (*a, *b))
    };

    let distance = edges()
        .map(|(a, b)| segment_distance2d(point, a, b))
        .fold(f64::INFINITY, f64::min);

    let winding: f64 = edges().map(|(a, b)| (a - point).angle_to(b - point)).sum();

    if winding.abs() > 1.0 {
        -distance
    } else {
        distance
    }
}
