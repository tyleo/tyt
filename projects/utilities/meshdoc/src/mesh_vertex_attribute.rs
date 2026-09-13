use crate::MeshAttributeComponents;

/// A further per-vertex attribute of a [`MeshPrimitive`](crate::MeshPrimitive)
/// outside the modeled streams.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshVertexAttribute {
    /// The attribute name, unique within its primitive.
    pub name: String,

    /// Components per vertex, at least `1`.
    pub width: usize,

    /// The components, `width` per vertex in vertex order.
    pub components: MeshAttributeComponents,
}
