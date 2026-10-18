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
    let edit_origin = object.origin().to_array();
    let centered = |size: u32| {
        let margin = dimension
            .checked_sub(size)
            .expect("the caller picked a cube holding the edit grid");
        (margin / 2) as i32
    };
    let canvas = [centered(edit.x), centered(edit.y), 0];
    let next = from_fn(|axis| canvas[axis] + placement.origin[axis] - edit_origin[axis]);
    for (axis, &coordinate) in next.iter().enumerate() {
        placement.center[axis] += (coordinate - placement.box_min[axis]) as f64;
    }
    placement.box_min = next;
    (tight, placement)
}

#[cfg(test)]
mod tests {
    use crate::place_object_in_workspace;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::VoxObject;

    /// A 300x10x4 canvas with one voxel at `(5, 3, 1)` and origin `(-2, 0, 0)`.
    fn canvas() -> VoxObject {
        let mut object = VoxObject::new("a".to_owned(), TyVector3U32::new(300, 10, 4)).unwrap();
        object.set_origin(TyVector3I32::new(-2, 0, 0));
        let voxel_id = object.voxel_id(TyVector3U32::new(5, 3, 1)).unwrap();
        object.retain_voxel(voxel_id, &[]).unwrap();
        object
    }

    #[test]
    fn centers_a_canvas_wider_than_256_in_its_cube() {
        let (_, placement) = place_object_in_workspace(&canvas(), 512);
        assert_eq!(placement.box_min, [106 + 5, 251 + 3, 1]);
        assert_eq!(placement.center, [111.5, 254.5, 1.5]);
    }

    #[test]
    #[should_panic(expected = "the caller picked a cube holding the edit grid")]
    fn a_cube_narrower_than_the_canvas_panics() {
        place_object_in_workspace(&canvas(), 256);
    }
}
