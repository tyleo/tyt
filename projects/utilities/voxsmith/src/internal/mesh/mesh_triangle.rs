use crate::MeshTriangleUvs;
use ty_math::TyVector3F64;

/// One mesh triangle in world space (Z-up), tagged with the material it was
/// drawn with. The tag is an index into the [`Mesh`](crate::Mesh) material
/// table, so the rasterizer can attribute each surface voxel to a material.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshTriangle {
    /// The triangle's three vertices.
    pub points: [TyVector3F64; 3],

    /// The per-vertex texture coordinates, one set per PBR map slot.
    pub uvs: MeshTriangleUvs,

    /// Index into the mesh's material table of the material this triangle uses.
    pub material: u32,
}
