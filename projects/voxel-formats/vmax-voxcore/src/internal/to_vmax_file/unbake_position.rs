use crate::ObjectPlacement;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};

/// Recovers an object's `t_p`, the inverse of the read path's
/// `object_transform`. It backs out the `t_p` Voxel Max renders with from the
/// node's transform, the content center it pivots about, and the grid `origin`:
/// `t_p = position - center - R*S* (box_min - center - origin)`. Uses the
/// axis-angle the object writes, so the two stay exact inverses.
pub(crate) fn unbake_position(
    transform: &TyTransformF64,
    rotation: TyQuaternionF64,
    placement: &ObjectPlacement,
) -> [f64; 3] {
    let center = placement.center;
    let box_min = placement.box_min;
    let origin = placement.origin;
    let scale = transform.scale;
    let offset = TyVector3F64::new(
        (box_min[0] as f64 - center[0] - origin[0] as f64) * scale.x,
        (box_min[1] as f64 - center[1] - origin[1] as f64) * scale.y,
        (box_min[2] as f64 - center[2] - origin[2] as f64) * scale.z,
    );
    let rotated = rotation * offset;
    [
        transform.position.x - center[0] - rotated.x,
        transform.position.y - center[1] - rotated.y,
        transform.position.z - center[2] - rotated.z,
    ]
}
