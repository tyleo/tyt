use crate::{
    Error, Result,
    operations::sdf_doc::{ArgumentCheck, EntryPaths, SdfShapes},
};
use branded_id::IteratorExt;
use sdfcore::{SdfShape3d, SdfState, SdfStep};
use std::f64::consts::PI;

/// Runs the sdfj format's validation rules that read the boxes in `shapes`.
/// An error starts with the path of the first step in `paths` that reaches the
/// failing entry.
pub fn check_shape_boxes(state: &SdfState, shapes: &SdfShapes, paths: &EntryPaths) -> Result<()> {
    for (shape3d_id, shape) in state.shapes3d.iter().enumerate_ids() {
        let checked = match shape {
            SdfShape3d::Bend {
                shape_id,
                along,
                radius,
                ..
            } => {
                let child = shapes
                    .bounds3d(*shape_id)
                    .expect("a bend's shape has a box");
                let length = child.max[along.index()] - child.min[along.index()];
                let most = PI * radius;

                ArgumentCheck::new("bend").expect(
                    length <= most,
                    "shape",
                    &format!("at most {most} long along {along} to bend within half a turn"),
                    length,
                )
            }

            SdfShape3d::Elongate {
                shape_id,
                lengths,
                center,
            } => match shapes.bounds3d(*shape_id) {
                Some(child) => {
                    let center = center.unwrap_or_default();

                    (0..3)
                        .filter(|axis| lengths[*axis] > 0.0)
                        .try_for_each(|axis| {
                            ArgumentCheck::new("elongate").expect(
                                (child.min[axis]..=child.max[axis]).contains(&center[axis]),
                                "center",
                                &format!(
                                    "inside its shape's box from {} to {} along {}",
                                    child.min[axis],
                                    child.max[axis],
                                    ["x", "y", "z"][axis],
                                ),
                                center[axis],
                            )
                        })
                }

                None => Ok(()),
            },

            _ => Ok(()),
        };

        checked.map_err(|message| {
            Error::invalid(format!("{}: {message}", paths.shape3d(shape3d_id)))
        })?;
    }

    for (step_id, step) in state.steps.iter().enumerate_ids() {
        if let SdfStep::Add { shape_id, .. } = step
            && shapes.bounds3d(*shape_id).is_none()
        {
            return Err(Error::invalid(format!(
                "{}: add shape must be bounded, not a shape that reaches without end",
                paths.step(step_id)
            )));
        }
    }

    Ok(())
}
