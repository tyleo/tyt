use crate::operations::voxelize::triangle_of;
use meshdoc::MeshPrimitive;
use ty_math::TyVector3F64;

/// An axis-aligned box spanning `[0, sx]`, `[0, sy]`, `[0, sz]`, indexed
/// triangles winding outward.
pub(crate) fn box_primitive(sx: f64, sy: f64, sz: f64) -> MeshPrimitive {
    let positions = vec![
        TyVector3F64::new(0.0, 0.0, 0.0),
        TyVector3F64::new(sx, 0.0, 0.0),
        TyVector3F64::new(sx, sy, 0.0),
        TyVector3F64::new(0.0, sy, 0.0),
        TyVector3F64::new(0.0, 0.0, sz),
        TyVector3F64::new(sx, 0.0, sz),
        TyVector3F64::new(sx, sy, sz),
        TyVector3F64::new(0.0, sy, sz),
    ];
    let faces = [
        [0, 1, 2],
        [0, 2, 3],
        [4, 6, 5],
        [4, 7, 6],
        [0, 4, 5],
        [0, 5, 1],
        [3, 2, 6],
        [3, 6, 7],
        [0, 3, 7],
        [0, 7, 4],
        [1, 5, 6],
        [1, 6, 2],
    ];

    MeshPrimitive::new(
        positions,
        faces
            .iter()
            .map(|face| triangle_of(face[0], face[1], face[2]))
            .collect(),
    )
    .unwrap()
}
