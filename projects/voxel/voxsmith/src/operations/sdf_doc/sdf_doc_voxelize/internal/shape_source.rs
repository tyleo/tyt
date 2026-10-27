use crate::operations::sdf_doc::SdfEvaluation;
use branded_id::U32Id;
use sdfcore::{BSdfShape2d, BSdfShape3d};
use ty_math::{TyVector2F64, TyVector3F64};

/// The shapes a field evaluates its children through.
pub trait ShapeSource {
    /// The 3D shape at `shape3d_id` evaluated at `point`.
    fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation;

    /// The signed distance of the 2D shape at `shape2d_id` at `point`.
    fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64;
}
