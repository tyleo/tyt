use crate::operations::sdf_doc::ArcSpan;
use ty_math::TyVector2F64;

/// The signed distance to the point at `offset` from the slice of the disk of
/// `radius` around the origin within `span`.
pub fn sector_distance(offset: TyVector2F64, radius: f64, span: &ArcSpan) -> f64 {
    let folded = span.fold(offset);
    let to_circle = folded.length() - radius;

    let end = span.end(1.0);
    let to_edge = (folded - end * folded.dot(end).clamp(0.0, radius)).length();

    to_circle.max(if span.is_past_end(folded) {
        to_edge
    } else {
        -to_edge
    })
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{arc_curve_distance, assert_shape2d, segment_distance2d};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    #[test]
    fn a_sector_measures_to_its_arc_and_its_edges() {
        let center = TyVector2F64::new(0.1, -0.1);
        let radius = 0.7;
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        for (from_degrees, to_degrees) in [(0.0, 90.0), (30.0, 300.0), (-45.0, 45.0), (0.0, 360.0)]
        {
            let end = |degrees: f64| {
                let (sin, cos) = f64::to_radians(degrees).sin_cos();
                center + TyVector2F64::new(cos, sin) * radius
            };

            let shape = SdfShape2d::Sector {
                center,
                radius,
                from_degrees,
                to_degrees,
            };

            assert_shape2d(vec![shape], grid, 1e-12, |point| {
                let to_outline =
                    arc_curve_distance(point, center, radius, from_degrees, to_degrees)
                        .min(segment_distance2d(point, center, end(from_degrees)))
                        .min(segment_distance2d(point, center, end(to_degrees)));

                let offset = point - center;
                let angle = offset.y.atan2(offset.x).to_degrees();
                let within = (angle - from_degrees).rem_euclid(360.0) <= to_degrees - from_degrees;

                if within && offset.length() <= radius {
                    -to_outline
                } else {
                    to_outline
                }
            });
        }
    }
}
