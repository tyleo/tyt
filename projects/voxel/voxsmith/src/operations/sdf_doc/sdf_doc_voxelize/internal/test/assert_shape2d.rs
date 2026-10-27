use crate::operations::sdf_doc::{bounds_of, grid2d, shapes_of};
use branded_id::U32Id;
use sdfcore::SdfShape2d;
use ty_math::TyVector2F64;

/// Checks the last of `shapes2d` over the grid from `min` to `max` with
/// `steps` points along each axis. Each distance lies within `tolerance` of
/// `expected`, and each point the shape covers lies inside the shape's box.
pub fn assert_shape2d(
    shapes2d: Vec<SdfShape2d>,
    (min, max, steps): (TyVector2F64, TyVector2F64, u32),
    tolerance: f64,
    expected: impl Fn(TyVector2F64) -> f64,
) {
    let shape2d_id = U32Id::from_u32(shapes2d.len() as u32 - 1);
    let bounds = bounds_of(&shapes2d, &[]).0[shape2d_id.to_usize_id()];
    let shapes = shapes_of(shapes2d, Vec::new());

    for point in grid2d(min, max, steps) {
        let distance = shapes.distance2d(shape2d_id, point);
        let expected = expected(point);

        assert!(
            (distance - expected).abs() <= tolerance,
            "at {point}: read {distance}, expected {expected}",
        );

        if distance <= 0.0 {
            assert!(
                point.cmpge(bounds.min).all() && point.cmple(bounds.max).all(),
                "{point} lies outside {bounds:?}",
            );
        }
    }
}
