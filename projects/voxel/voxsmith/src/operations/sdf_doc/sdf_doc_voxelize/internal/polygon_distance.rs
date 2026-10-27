use ty_math::TyVector2F64;

/// The signed distance to `point` from the polygon whose outline runs through
/// `points` and closes back to the first. The sign follows the even-odd rule.
pub fn polygon_distance(points: &[TyVector2F64], point: TyVector2F64) -> f64 {
    let mut distance_squared = f64::INFINITY;
    let mut inside = false;

    for (index, &vertex) in points.iter().enumerate() {
        let previous = points[(index + points.len() - 1) % points.len()];
        let e = previous - vertex;
        let w = point - vertex;

        // A lathe's outline repeats each point that lies on the axis, which
        // leaves an edge with no length.
        let e_squared = e.dot(e);
        let t = if e_squared > 0.0 {
            (w.dot(e) / e_squared).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let b = w - e * t;
        distance_squared = distance_squared.min(b.dot(b));

        let tests = [
            point.y >= vertex.y,
            point.y < previous.y,
            e.x * w.y > e.y * w.x,
        ];
        if tests == [true; 3] || tests == [false; 3] {
            inside = !inside;
        }
    }

    let distance = distance_squared.sqrt();

    if inside { -distance } else { distance }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape2d, polygon_distance, polygon_reference};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    fn arrow() -> Vec<TyVector2F64> {
        [
            (-0.8, -0.2),
            (0.2, -0.2),
            (0.2, -0.6),
            (0.9, 0.0),
            (0.2, 0.6),
            (0.2, 0.2),
            (-0.8, 0.2),
        ]
        .map(|(u, v)| TyVector2F64::new(u, v))
        .to_vec()
    }

    #[test]
    fn a_polygon_measures_exact_distances_around_its_concave_corners() {
        let points = arrow();
        let shape = SdfShape2d::Polygon {
            points: points.clone(),
        };
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_shape2d(vec![shape], grid, 1e-12, |point| {
            polygon_reference(&points, point)
        });
    }

    #[test]
    fn a_repeated_point_leaves_the_distance_alone() {
        let points = arrow();
        let mut repeated = points.clone();
        repeated.insert(3, repeated[3]);

        for point in [
            TyVector2F64::new(0.5, 0.0),
            TyVector2F64::new(0.95, 0.05),
            TyVector2F64::new(-0.9, 0.3),
        ] {
            assert_eq!(
                polygon_distance(&repeated, point),
                polygon_distance(&points, point)
            );
        }
    }
}
