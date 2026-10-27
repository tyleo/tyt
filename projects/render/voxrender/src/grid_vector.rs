use ty_math::{TyTransformF64, TyVector3F64};

/// `vector` in world meters, carried into the grid of a placement under
/// `transform`.
pub fn grid_vector(transform: &TyTransformF64, vector: TyVector3F64) -> TyVector3F64 {
    (transform.rotation.inverse() * vector) / transform.scale
}
