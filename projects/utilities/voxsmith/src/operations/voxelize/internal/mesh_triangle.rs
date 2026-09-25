use branded_id::U32Id;
use meshdoc::BMeshVertex;
use ty_math::TyVector3F64;

/// One mesh triangle in world space, tagged with its placed primitive. The
/// tag indexes the [`MeshInput`] primitive table, where the material and the
/// UV streams live.
///
/// [`MeshInput`]: crate::operations::voxelize::MeshInput
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshTriangle {
    /// The triangle's three vertices.
    pub points: [TyVector3F64; 3],

    /// The three vertices' ids in the placed primitive.
    pub vertex_ids: [U32Id<BMeshVertex>; 3],

    /// Index into the mesh's primitive table.
    pub primitive: u32,
}
