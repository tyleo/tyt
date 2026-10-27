use ty_math::TyVector2F64;

/// The signed distance to `point` from the arch between `min` and `max`: a
/// rectangle topped with a half circle as wide as the rectangle.
pub fn arch_distance(point: TyVector2F64, min: TyVector2F64, max: TyVector2F64) -> f64 {
    let half_width = (max.x - min.x) / 2.0;
    let center = TyVector2F64::new(min.x + half_width, max.y - half_width);
    let height = center.y - min.y;

    let p = point - center;
    let p = TyVector2F64::new(p.x.abs(), -p.y);
    let q = p - TyVector2F64::new(half_width, height);

    let to_base = TyVector2F64::new(q.x.max(0.0), q.y).length_squared();
    let to_wall_x = if p.y > 0.0 {
        q.x
    } else {
        p.length() - half_width
    };
    let to_wall = TyVector2F64::new(to_wall_x, q.y.max(0.0)).length_squared();

    let distance = to_base.min(to_wall).sqrt();

    if to_wall_x.max(q.y) < 0.0 {
        -distance
    } else {
        distance
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{arc_curve_distance, assert_shape2d, segment_distance2d};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    #[test]
    fn an_arch_measures_to_its_base_sides_and_half_circle() {
        let min = TyVector2F64::new(-0.5, -0.6);
        let max = TyVector2F64::new(0.5, 0.8);
        let center = TyVector2F64::new(0.0, 0.3);
        let corner = |u: f64, v: f64| TyVector2F64::new(u, v);

        let shape = SdfShape2d::Arch { min, max };
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_shape2d(vec![shape], grid, 1e-12, |point| {
            let to_outline = [
                segment_distance2d(point, corner(-0.5, -0.6), corner(0.5, -0.6)),
                segment_distance2d(point, corner(-0.5, -0.6), corner(-0.5, 0.3)),
                segment_distance2d(point, corner(0.5, -0.6), corner(0.5, 0.3)),
                arc_curve_distance(point, center, 0.5, 0.0, 180.0),
            ]
            .into_iter()
            .fold(f64::INFINITY, f64::min);

            let in_rect = point.x.abs() <= 0.5 && (-0.6..=0.3).contains(&point.y);
            let in_top = point.y >= 0.3 && point.distance(center) <= 0.5;

            if in_rect || in_top {
                -to_outline
            } else {
                to_outline
            }
        });
    }
}
