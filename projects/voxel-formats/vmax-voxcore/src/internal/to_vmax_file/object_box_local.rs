use crate::tighten;
use branded_id::U32Id;
use ty_math::TyBoundsF64;
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// An object's content box `(center, half)` in its placing node's local voxel
/// frame: the tight runtime grid `[origin, origin + bounds]`. An empty object
/// has no runtime extent of its own, so it frames its build volume instead,
/// matching the content box the write path gives it.
pub(crate) fn object_box_local<T: VoxExt>(
    main: &VoxMain<T>,
    object_id: U32Id<BVoxObject>,
) -> ([f64; 3], [f64; 3]) {
    let object = main.object(object_id).expect("a valid child object");
    let (tight, (edit_bounds, edit_origin)) = tighten(object);
    let bounds = tight.bounds();
    let box_local = if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        TyBoundsF64::from_min_size(edit_origin.as_dvec3(), edit_bounds.as_dvec3())
    } else {
        TyBoundsF64::from_min_size(tight.origin().as_dvec3(), bounds.as_dvec3())
    };
    (box_local.center.to_array(), box_local.extents.to_array())
}
