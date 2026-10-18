use crate::{ObjectPlacement, place_object};
use std::array::from_fn;
use voxcore::VoxObject;

/// Places an object's canvas in the chosen editor cube, centered in x and y
/// and on the z floor. The caller has checked that the cube contains the
/// canvas.
pub fn place_object_in_workspace(
    object: &VoxObject,
    dimension: u32,
) -> (VoxObject, ObjectPlacement) {
    let (tight, mut placement) = place_object(object);
    let edit = object.bounds();
    let centered = |size: u32, width: u32| ((width as i64 - size as i64) / 2).max(0) as i32;
    let previous = [centered(edit.x, 256), centered(edit.y, 256), 0];
    let canvas = [centered(edit.x, dimension), centered(edit.y, dimension), 0];
    let next = from_fn(|axis| placement.box_min[axis] + canvas[axis] - previous[axis]);
    for (axis, &coordinate) in next.iter().enumerate() {
        placement.center[axis] += (coordinate - placement.box_min[axis]) as f64;
    }
    placement.box_min = next;
    (tight, placement)
}
