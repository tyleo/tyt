use crate::operations::sdf_doc::{Bounds2d, Bounds3d, Shape2dField, Shape3dField};
use branded_id::IdVec;
use sdfcore::{BSdfShape2d, BSdfShape3d, SdfShape2d, SdfShape3d};

/// The boxes of `shapes2d` and `shapes3d`, computed as `SdfShapes::new`
/// computes them.
pub fn bounds_of(
    shapes2d: &[SdfShape2d],
    shapes3d: &[SdfShape3d],
) -> (
    IdVec<BSdfShape2d, Bounds2d>,
    IdVec<BSdfShape3d, Option<Bounds3d>>,
) {
    let mut bounds2d = IdVec::new();

    for shape in shapes2d {
        let bounds = Shape2dField::new(shape).bounds(&bounds2d);
        bounds2d.push(bounds);
    }

    let mut bounds3d = IdVec::new();

    for shape in shapes3d {
        let bounds = Shape3dField::new(shape, &bounds3d).bounds(&bounds3d, &bounds2d);
        bounds3d.push(bounds);
    }

    (bounds2d, bounds3d)
}
