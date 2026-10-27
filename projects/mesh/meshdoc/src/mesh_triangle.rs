use crate::BMeshVertex;
use branded_id::U32Id;

/// One triangle of a [`MeshPrimitive`](crate::MeshPrimitive), wound
/// counter-clockwise seen from outside.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MeshTriangle {
    /// The corners, in winding order.
    pub vertex_ids: [U32Id<BMeshVertex>; 3],
}
