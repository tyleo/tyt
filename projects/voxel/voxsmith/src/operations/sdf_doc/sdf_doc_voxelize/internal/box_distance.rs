use ty_math::TyVector3F64;

/// The signed distance to the point at `offset` from the box with
/// `half_extents` around the origin.
pub fn box_distance(offset: TyVector3F64, half_extents: TyVector3F64) -> f64 {
    let q = offset.abs() - half_extents;
    q.max(TyVector3F64::ZERO).length() + q.max_element().min(0.0)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape3d, box_reference};
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    const MIN: TyVector3F64 = TyVector3F64::new(-0.5, 0.0, -0.25);

    const MAX: TyVector3F64 = TyVector3F64::new(0.5, 0.75, 0.25);

    const GRID: (TyVector3F64, TyVector3F64, u32) =
        (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

    #[test]
    fn a_box_measures_exact_distances() {
        let shape = SdfShape3d::Box {
            min: MIN,
            max: MAX,
            round: None,
        };

        assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
            box_reference(point, MIN, MAX)
        });
    }

    #[test]
    fn a_box_measures_the_same_with_its_corners_swapped_per_axis() {
        let shape = SdfShape3d::Box {
            min: TyVector3F64::new(MAX.x, MIN.y, MAX.z),
            max: TyVector3F64::new(MIN.x, MAX.y, MIN.z),
            round: None,
        };

        assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
            box_reference(point, MIN, MAX)
        });
    }

    #[test]
    fn a_round_box_rounds_inside_its_corners() {
        let round = 0.125;
        let shape = SdfShape3d::Box {
            min: MIN,
            max: MAX,
            round: Some(round),
        };

        assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
            box_reference(point, MIN + round, MAX - round) - round
        });
    }
}
