use ty_math::TyVector2F64;

/// The signed distance to `point` from the pointed lens from `a` to `b` that
/// reaches `half_width` to each side of its middle.
pub fn vesica_distance(
    point: TyVector2F64,
    a: TyVector2F64,
    b: TyVector2F64,
    half_width: f64,
) -> f64 {
    let r = 0.5 * (b - a).length();
    let d = 0.5 * (r * r - half_width * half_width) / half_width;
    let v = (b - a) / r;
    let p = point - (b + a) * 0.5;
    let q = TyVector2F64::new(v.y * p.x - v.x * p.y, v.x * p.x + v.y * p.y).abs() * 0.5;

    if r * q.x < d * (q.y - r) {
        (q - TyVector2F64::new(0.0, r)).length()
    } else {
        (q - TyVector2F64::new(-d, 0.0)).length() - (d + half_width)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{arc_curve_distance, assert_shape2d};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    #[test]
    fn a_vesica_measures_to_its_two_arcs() {
        let a = TyVector2F64::new(-0.6, -0.2);
        let b = TyVector2F64::new(0.5, 0.4);
        let half_width = 0.25;

        // The lens is the overlap of two disks whose circles pass through both
        // tips and reach `half_width` from the middle.
        let half_length = a.distance(b) / 2.0;
        let middle = (a + b) / 2.0;
        let across = (b - a).normalize().perp();
        let shift = (half_length * half_length - half_width * half_width) / (2.0 * half_width);
        let circle_radius = shift + half_width;

        let angle = |point: TyVector2F64, center: TyVector2F64| {
            let offset = point - center;
            offset.y.atan2(offset.x).to_degrees()
        };

        // The arc of the circle about `center` from tip to tip through
        // `through`.
        let side = |center: TyVector2F64, through: TyVector2F64| {
            let (from_a, from_b) = (angle(a, center), angle(b, center));
            let through = angle(through, center);

            if (through - from_a).rem_euclid(360.0) < (from_b - from_a).rem_euclid(360.0) {
                (center, from_a, from_a + (from_b - from_a).rem_euclid(360.0))
            } else {
                (center, from_b, from_b + (from_a - from_b).rem_euclid(360.0))
            }
        };

        let sides = [
            side(middle - across * shift, middle + across * half_width),
            side(middle + across * shift, middle - across * half_width),
        ];

        let shape = SdfShape2d::Vesica {
            a,
            b,
            width: half_width * 2.0,
        };
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_shape2d(vec![shape], grid, 1e-12, |point| {
            let to_outline = sides
                .iter()
                .map(|(center, from, to)| {
                    arc_curve_distance(point, *center, circle_radius, *from, *to)
                })
                .fold(f64::INFINITY, f64::min);
            let inside = sides
                .iter()
                .all(|(center, ..)| point.distance(*center) <= circle_radius);

            if inside { -to_outline } else { to_outline }
        });
    }
}
