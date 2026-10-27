use ty_math::TyVector2F64;

/// The signed distance to a point from the round cone with radius `radius_a`
/// at its first end and `radius_b` at its second:
///
/// 1. `radial` holds the point's distance from the axis
/// 2. `along` holds the point's position along the axis from the first end
/// 3. `length` holds the distance between the ends
pub fn round_cone_distance(
    radial: f64,
    along: f64,
    radius_a: f64,
    radius_b: f64,
    length: f64,
) -> f64 {
    let b = (radius_a - radius_b) / length;
    let a = (1.0 - b * b).sqrt();
    let k = radial * -b + along * a;

    if k < 0.0 {
        TyVector2F64::new(radial, along).length() - radius_a
    } else if k > a * length {
        TyVector2F64::new(radial, along - length).length() - radius_b
    } else {
        radial * a + along * b - radius_a
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::assert_shape3d;
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    /// The distance to the round cone as the union of the spheres along its
    /// axis, with the radius running evenly from end to end. The union's
    /// distance runs convex along the axis, and a ternary search finds its
    /// least value.
    fn round_cone_reference(
        point: TyVector3F64,
        a: TyVector3F64,
        b: TyVector3F64,
        radius_a: f64,
        radius_b: f64,
    ) -> f64 {
        let sphere = |t: f64| point.distance(a.lerp(b, t)) - (radius_a + (radius_b - radius_a) * t);
        let (mut low, mut high) = (0.0, 1.0);

        for _ in 0..200 {
            let first = low + (high - low) / 3.0;
            let second = high - (high - low) / 3.0;

            if sphere(first) < sphere(second) {
                high = second;
            } else {
                low = first;
            }
        }

        sphere((low + high) / 2.0)
    }

    #[test]
    fn a_round_cone_measures_exact_distances_at_any_angle() {
        let a = TyVector3F64::new(0.1, -0.4, 0.2);
        let b = TyVector3F64::new(-0.3, 0.5, 0.1);
        let grid = (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

        for (radius_a, radius_b) in [(0.4, 0.15), (0.1, 0.3), (0.25, 0.25)] {
            let shape = SdfShape3d::RoundCone {
                a,
                b,
                radius_a,
                radius_b,
            };

            assert_shape3d(Vec::new(), vec![shape], grid, 1e-9, |point| {
                round_cone_reference(point, a, b, radius_a, radius_b)
            });
        }
    }
}
