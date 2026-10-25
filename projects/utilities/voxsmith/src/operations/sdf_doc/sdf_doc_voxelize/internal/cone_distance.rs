/// The signed distance to a point from the cone between end radii `radius_a`
/// and `radius_b`:
///
/// 1. `radial` holds the point's distance from the axis
/// 2. `t` holds the point's position along the axis, 0 at end a and 1 at end b
/// 3. `length_squared` holds the squared distance between the ends
pub fn cone_distance(
    radial: f64,
    t: f64,
    length_squared: f64,
    radius_a: f64,
    radius_b: f64,
) -> f64 {
    let rba = radius_b - radius_a;
    let cax = (radial - if t < 0.5 { radius_a } else { radius_b }).max(0.0);
    let cay = (t - 0.5).abs() - 0.5;
    let k = rba * rba + length_squared;
    let f = ((rba * (radial - radius_a) + t * length_squared) / k).clamp(0.0, 1.0);
    let cbx = radial - radius_a - f * rba;
    let cby = t - f;

    let distance = (cax * cax + cay * cay * length_squared)
        .min(cbx * cbx + cby * cby * length_squared)
        .sqrt();

    if cbx < 0.0 && cay < 0.0 {
        -distance
    } else {
        distance
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape3d, polygon_reference};
    use sdfcore::SdfShape3d;
    use ty_math::{TyVector2F64, TyVector3F64};

    /// The distance to the cone through the trapezoid it sweeps in the
    /// half-plane through its axis.
    fn cone_reference(
        point: TyVector3F64,
        a: TyVector3F64,
        b: TyVector3F64,
        radius_a: f64,
        radius_b: f64,
    ) -> f64 {
        let axis = (b - a).normalize();
        let length = (b - a).length();
        let offset = point - a;
        let trapezoid = [
            TyVector2F64::new(-radius_a, 0.0),
            TyVector2F64::new(radius_a, 0.0),
            TyVector2F64::new(radius_b, length),
            TyVector2F64::new(-radius_b, length),
        ];

        polygon_reference(
            &trapezoid,
            TyVector2F64::new(offset.cross(axis).length(), offset.dot(axis)),
        )
    }

    #[test]
    fn a_cone_measures_exact_distances_at_any_angle() {
        let a = TyVector3F64::new(0.1, -0.4, 0.2);
        let b = TyVector3F64::new(-0.3, 0.6, 0.1);
        let grid = (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

        for (radius_a, radius_b) in [(0.5, 0.2), (0.1, 0.45), (0.4, 0.0)] {
            let shape = SdfShape3d::Cone {
                a,
                b,
                radius_a,
                radius_b,
            };

            assert_shape3d(Vec::new(), vec![shape], grid, 1e-9, |point| {
                cone_reference(point, a, b, radius_a, radius_b)
            });
        }
    }
}
