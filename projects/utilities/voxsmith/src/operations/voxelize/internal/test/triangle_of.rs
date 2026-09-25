use branded_id::U32Id;
use meshdoc::MeshTriangle;

/// A triangle over three vertex indices.
pub(crate) fn triangle_of(a: u32, b: u32, c: u32) -> MeshTriangle {
    MeshTriangle {
        vertex_ids: [U32Id::from_u32(a), U32Id::from_u32(b), U32Id::from_u32(c)],
    }
}
