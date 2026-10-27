use crate::operations::sdf_doc::{bounds_of, grid3d, shapes_of};
use branded_id::U32Id;
use sdfcore::{SdfShape2d, SdfShape3d};
use ty_math::TyVector3F64;

/// Checks the last of `shapes3d` over the grid from `min` to `max` with
/// `steps` points along each axis. Each distance lies within `tolerance` of
/// `expected`, and each point the shape covers lies inside the shape's box.
pub fn assert_shape3d(
    shapes2d: Vec<SdfShape2d>,
    shapes3d: Vec<SdfShape3d>,
    (min, max, steps): (TyVector3F64, TyVector3F64, u32),
    tolerance: f64,
    expected: impl Fn(TyVector3F64) -> f64,
) {
    let shape3d_id = U32Id::from_u32(shapes3d.len() as u32 - 1);
    let bounds = bounds_of(&shapes2d, &shapes3d).1[shape3d_id.to_usize_id()];
    let shapes = shapes_of(shapes2d, shapes3d);

    for point in grid3d(min, max, steps) {
        let distance = shapes.evaluate(shape3d_id, point).distance;
        let expected = expected(point);

        assert!(
            (distance - expected).abs() <= tolerance,
            "at {point}: read {distance}, expected {expected}",
        );

        if distance <= 0.0
            && let Some(bounds) = bounds
        {
            assert!(
                point.cmpge(bounds.min).all() && point.cmple(bounds.max).all(),
                "{point} lies outside {bounds:?}",
            );
        }
    }
}
