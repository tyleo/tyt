use ty_math::TyVector2F64;

/// The signed distance to `point` from the line through `points` stroked
/// `half_width` to each side.
pub fn polyline_distance(points: &[TyVector2F64], half_width: f64, point: TyVector2F64) -> f64 {
    let distance_squared = points
        .windows(2)
        .map(|segment| {
            let e = segment[1] - segment[0];
            let w = point - segment[0];
            let b = w - e * (w.dot(e) / e.dot(e)).clamp(0.0, 1.0);
            b.dot(b)
        })
        .fold(f64::INFINITY, f64::min);

    distance_squared.sqrt() - half_width
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape2d, segment_distance2d};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    #[test]
    fn a_polyline_strokes_its_segments() {
        let points: Vec<TyVector2F64> = [(-0.8, -0.5), (-0.2, 0.4), (0.3, -0.3), (0.8, 0.5)]
            .map(|(u, v)| TyVector2F64::new(u, v))
            .to_vec();
        let width = 0.2;
        let shape = SdfShape2d::Polyline {
            points: points.clone(),
            width,
        };
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_shape2d(vec![shape], grid, 1e-12, |point| {
            points
                .windows(2)
                .map(|segment| segment_distance2d(point, segment[0], segment[1]))
                .fold(f64::INFINITY, f64::min)
                - width / 2.0
        });
    }
}
