use ty_math::TyVector2F64;

/// The signed distance to the point at `offset` from the rectangle with
/// `half_extents` around the origin.
pub fn rect_distance(offset: TyVector2F64, half_extents: TyVector2F64) -> f64 {
    let q = offset.abs() - half_extents;
    q.max(TyVector2F64::ZERO).length() + q.max_element().min(0.0)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape2d, rect_reference};
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    const MIN: TyVector2F64 = TyVector2F64::new(-0.5, -0.25);

    const MAX: TyVector2F64 = TyVector2F64::new(0.75, 0.5);

    const GRID: (TyVector2F64, TyVector2F64, u32) =
        (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

    #[test]
    fn a_rect_measures_exact_distances() {
        let shape = SdfShape2d::Rect {
            min: MIN,
            max: MAX,
            chamfer: None,
            round: None,
        };

        assert_shape2d(vec![shape], GRID, 1e-12, |point| {
            rect_reference(point, MIN, MAX)
        });
    }

    #[test]
    fn a_round_rect_rounds_inside_its_corners() {
        let round = 0.2;
        let shape = SdfShape2d::Rect {
            min: MIN,
            max: MAX,
            chamfer: None,
            round: Some(round),
        };

        assert_shape2d(vec![shape], GRID, 1e-12, |point| {
            rect_reference(point, MIN + round, MAX - round) - round
        });
    }
}
