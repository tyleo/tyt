use std::f64::consts::FRAC_1_SQRT_2;
use ty_math::TyVector2F64;

/// The signed distance to the point at `offset` from the rectangle with
/// `half_extents` around the origin whose corners are cut at 45 degrees with
/// legs `chamfer` long.
pub fn chamfer_rect_distance(
    offset: TyVector2F64,
    half_extents: TyVector2F64,
    chamfer: f64,
) -> f64 {
    let p = offset.abs() - half_extents;
    let p = if p.y > p.x {
        TyVector2F64::new(p.y, p.x)
    } else {
        p
    };
    let p = TyVector2F64::new(p.x, p.y + chamfer);

    let k = 1.0 - 2.0_f64.sqrt();

    if p.y < 0.0 && p.y + p.x * k < 0.0 {
        return p.x;
    }

    if p.x < p.y {
        return (p.x + p.y) * FRAC_1_SQRT_2;
    }

    p.length()
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape2d, polygon_reference};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    #[test]
    fn a_chamfer_cuts_each_corner_at_45_degrees() {
        let min = TyVector2F64::new(-0.5, -0.25);
        let max = TyVector2F64::new(0.75, 0.5);
        let chamfer = 0.2;
        let octagon = [
            TyVector2F64::new(min.x + chamfer, min.y),
            TyVector2F64::new(max.x - chamfer, min.y),
            TyVector2F64::new(max.x, min.y + chamfer),
            TyVector2F64::new(max.x, max.y - chamfer),
            TyVector2F64::new(max.x - chamfer, max.y),
            TyVector2F64::new(min.x + chamfer, max.y),
            TyVector2F64::new(min.x, max.y - chamfer),
            TyVector2F64::new(min.x, min.y + chamfer),
        ];

        let shape = SdfShape2d::Rect {
            min,
            max,
            chamfer: Some(chamfer),
            round: None,
        };
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_shape2d(vec![shape], grid, 1e-12, |point| {
            polygon_reference(&octagon, point)
        });
    }
}
