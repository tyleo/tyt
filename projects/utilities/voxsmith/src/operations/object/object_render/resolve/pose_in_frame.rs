use crate::operations::object::{Rotation, resolve_rotation};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};

/// The world position and rotation of an entity posed at `position` with
/// `rotation` on `frame`'s axes. The rotation is `None` when a look-at aims
/// at `position` itself.
pub fn pose_in_frame(
    frame: &TyTransformF64,
    position: TyVector3F64,
    rotation: &Rotation,
) -> (TyVector3F64, Option<TyQuaternionF64>) {
    (
        frame.transform_point(position),
        resolve_rotation(rotation, position).map(|rotation| frame.rotation * rotation),
    )
}
