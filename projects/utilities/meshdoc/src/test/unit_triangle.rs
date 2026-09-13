use crate::{MeshPrimitive, MeshTriangle};
use branded_id::U32Id;
use ty_math::TyVector3F64;

/// A primitive of one triangle over three vertices, with no other streams
/// and no material.
pub fn unit_triangle() -> MeshPrimitive {
    MeshPrimitive::new(
        vec![TyVector3F64::ZERO, TyVector3F64::X, TyVector3F64::Y],
        vec![MeshTriangle {
            vertex_ids: [U32Id::from_u32(0), U32Id::from_u32(1), U32Id::from_u32(2)],
        }],
    )
    .expect("three finite vertices and one triangle over them")
}
