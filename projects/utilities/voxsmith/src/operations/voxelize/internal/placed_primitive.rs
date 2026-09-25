use meshdoc::{MeshMaterial, MeshPrimitive};

/// One primitive as the hierarchy places it. Its triangles ride the mesh's
/// triangle table in world space, tagged with this placement.
pub(crate) struct PlacedPrimitive<'a> {
    /// The document primitive.
    pub primitive: &'a MeshPrimitive,

    /// The material the primitive draws, the default when it has none.
    pub material: &'a MeshMaterial,
}
