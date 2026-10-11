use crate::{ObjectPlacement, place_object};
use std::array::from_fn;
use voxcore::VoxObject;

/// Places an object in the chosen editor cube, centered in x and y and on
/// the z floor. A fixed size centers the live grid, discarding empty canvas
/// margins; automatic selection retains those margins. The caller has checked
/// that the chosen cube contains the relevant grid.
pub fn place_object_in_workspace(
    object: &VoxObject,
    dimension: u32,
    compact: bool,
) -> (VoxObject, ObjectPlacement) {
    let (tight, mut placement) = place_object(object);
    let edit = object.bounds();
    let has_live = tight.bounds().x > 0;
    let centered = |size: u32, width: u32| ((width as i64 - size as i64) / 2).max(0) as i32;
    let previous = [centered(edit.x, 256), centered(edit.y, 256), 0];
    let bounds = if compact && has_live {
        tight.bounds()
    } else {
        edit
    };
    let canvas = [
        centered(bounds.x, dimension),
        centered(bounds.y, dimension),
        0,
    ];
    let next = if compact && has_live {
        canvas
    } else {
        from_fn(|axis| placement.box_min[axis] + canvas[axis] - previous[axis])
    };
    for (axis, &coordinate) in next.iter().enumerate() {
        placement.center[axis] += (coordinate - placement.box_min[axis]) as f64;
    }
    placement.box_min = next;
    (tight, placement)
}
