use ty_math::{TyTransformF64, TyVector3F64};

/// `point` in world meters, carried into the grid of a placement under
/// `transform`.
pub fn grid_point(transform: &TyTransformF64, point: TyVector3F64) -> TyVector3F64 {
    (transform.rotation.inverse() * (point - transform.position)) / transform.scale
}
