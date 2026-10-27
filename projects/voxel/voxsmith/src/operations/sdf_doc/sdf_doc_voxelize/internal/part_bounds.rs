use crate::operations::sdf_doc::{Bounds3d, SdfShapes};
use branded_id::U32Id;
use sdfcore::{BSdfObject, SdfState, SdfStep};

/// The box around the `add` shapes and `set` cubes of the object at
/// `object_id`, before any rounding, or `None` for an object with neither.
pub fn part_bounds(
    state: &SdfState,
    shapes: &SdfShapes,
    object_id: U32Id<BSdfObject>,
) -> Option<Bounds3d> {
    state.objects[object_id.to_usize_id()]
        .step_ids
        .iter()
        .flat_map(|step_id| match &state.steps[step_id.to_usize_id()] {
            SdfStep::Add { shape_id, .. } => {
                vec![shapes.bounds3d(*shape_id).expect("an add shape has a box")]
            }

            SdfStep::Set { points, size, .. } => points
                .iter()
                .map(|point| Bounds3d {
                    min: point - size / 2.0,
                    max: point + size / 2.0,
                })
                .collect(),

            SdfStep::Carve { .. } | SdfStep::Coat { .. } | SdfStep::Paint { .. } => Vec::new(),
        })
        .reduce(|bounds, step| bounds.union(&step))
}
