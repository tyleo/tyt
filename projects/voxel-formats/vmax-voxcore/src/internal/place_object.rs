use crate::{ObjectPlacement, tighten};
use ty_math::{TyBoundsF64, TyVector3I32, TyVector3U32};
use vmax::VMaxViewBox;
use voxcore::VoxObject;

/// Tightens `object` to its live voxels and derives its internal-grid
/// placement for the Voxel Max write.
pub fn place_object(object: &VoxObject) -> (VoxObject, ObjectPlacement) {
    let (tight, (edit_bounds, edit_origin)) = tighten(object);
    let placement = object_placement(tight.bounds(), tight.origin(), edit_bounds, edit_origin);
    (tight, placement)
}

/// The build volume's edit-space origin for a synthesized object grid of
/// `size`: centered in the 256-wide workspace in x and y and on the floor in z,
/// where Voxel Max frames a fresh object. A model wider than the workspace
/// keeps the origin corner so its voxels stay non-negative.
fn centered_origin(size: TyVector3U32) -> [i32; 3] {
    let centered = |s: u32| ((256 - s as i64) / 2).max(0) as i32;
    [centered(size.x), centered(size.y), 0]
}

/// Derives an object's internal-grid placement by convention. The canvas, the
/// edit grid, is centered in the 256-wide workspace, and the runtime grid sits
/// inside it by the runtime/edit origin offset. The content box follows from
/// the tight bounds. The internal-grid position is invisible to the scene,
/// which the node transform places, so re-deriving it loses nothing.
fn object_placement(
    bounds: TyVector3U32,
    origin: TyVector3I32,
    edit_bounds: TyVector3U32,
    edit_origin: TyVector3I32,
) -> ObjectPlacement {
    let canvas_min = centered_origin(edit_bounds);
    let origin = [origin.x, origin.y, origin.z];
    let box_min = [
        canvas_min[0] + origin[0] - edit_origin.x,
        canvas_min[1] + origin[1] - edit_origin.y,
        canvas_min[2] + origin[2] - edit_origin.z,
    ];
    // An empty runtime grid has no content box. Framing it on the build volume
    // keeps Voxel Max's content center and box sensible.
    let (content_min, content_size) = if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        (canvas_min, edit_bounds)
    } else {
        (box_min, bounds)
    };
    let (center, bounds_min, bounds_max) = content_box(content_min, content_size);
    ObjectPlacement {
        box_min,
        origin,
        center,
        bounds_min,
        bounds_max,
        view_box: object_view_box(canvas_min, edit_bounds),
    }
}

/// The Voxel Max content bounds `(e_c, e_mi, e_ma)` for a grid of `bounds`
/// whose min corner sits at `box_min`: the box center and its symmetric
/// half-extents. Voxel Max renders and frames against this and pivots about the
/// center.
fn content_box(box_min: [i32; 3], bounds: TyVector3U32) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let box_local = TyBoundsF64::from_min_size(
        TyVector3I32::from_array(box_min).as_dvec3(),
        bounds.as_dvec3(),
    );
    (
        box_local.center.to_array(),
        (-box_local.extents).to_array(),
        box_local.extents.to_array(),
    )
}

/// The `tools.vp` partition box for an object: its build volume at `origin`,
/// inclusive, so Voxel Max frames the whole authored grid.
fn object_view_box(origin: [i32; 3], size: TyVector3U32) -> VMaxViewBox {
    VMaxViewBox {
        min: [origin[0] as i64, origin[1] as i64, origin[2] as i64],
        max: [
            origin[0] as i64 + size.x.max(1) as i64 - 1,
            origin[1] as i64 + size.y.max(1) as i64 - 1,
            origin[2] as i64 + size.z.max(1) as i64 - 1,
        ],
    }
}
