use crate::decode_axis_angle;
use ty_math::{TyTransformF64, TyVector3F64, TyVector3I32};
use vmax::VMaxObject;

/// The node transform that places an object so rotating the node pivots its
/// grid about the content center. Voxel Max renders a voxel at `t_p + center +
/// R*S*(voxel - center)`, and a voxel is `box_min + local`, which sits at
/// node-local `origin + local`, so the node position is `t_p + center +
/// R*S*(box_min - center - origin)`. The bracket is the sub-voxel remainder
/// `box_min - center - origin`, so the position lands on the pivot and
/// rendering stays exact for any integer `origin`.
pub(crate) fn object_transform(
    object: &VMaxObject,
    box_min: [i32; 3],
    origin: [i32; 3],
) -> TyTransformF64 {
    let rotation = decode_axis_angle(object.rotation);
    let scale = TyVector3F64::from_array(object.scale);
    let center = TyVector3F64::from_array(object.center);
    let box_min = TyVector3I32::from_array(box_min).as_dvec3();
    let origin = TyVector3I32::from_array(origin).as_dvec3();

    // t_p + center + R*S*(box_min - center - origin); the bracket is the
    // sub-voxel remainder.
    let offset = (box_min - center - origin) * scale;
    let position = TyVector3F64::from_array(object.position) + center + rotation * offset;

    TyTransformF64::new(position, rotation, scale)
}
