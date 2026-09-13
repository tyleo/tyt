use crate::decode_axis_angle;
use ty_math::{TyTransformF64, TyVector3F64};
use vmax::VMaxGroup;

/// The transform for a scene group, placed directly at its authored position.
pub(crate) fn group_transform(group: &VMaxGroup) -> TyTransformF64 {
    TyTransformF64::new(
        TyVector3F64::from_array(group.position),
        decode_axis_angle(group.rotation),
        TyVector3F64::from_array(group.scale),
    )
}
