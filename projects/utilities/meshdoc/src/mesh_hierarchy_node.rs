use crate::{BMeshHierarchyNode, BMeshObject};
use branded_id::U32Id;
use ty_math::TyTransformF64;

/// A node in the document hierarchy.
///
/// Nodes form a DAG: a node may have several parents, so the same node can be
/// reused across the document. Within one node, each direct child node and
/// child object appears at most once. The ids reference a
/// [`MeshMain`](crate::MeshMain) and are meaningful only within it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshHierarchyNode {
    /// Display name.
    pub name: String,

    /// Transform applied to this node and its subtree, in meters, Z-up.
    pub transform: TyTransformF64,

    /// Child nodes.
    pub child_node_ids: Vec<U32Id<BMeshHierarchyNode>>,

    /// Objects placed by this node.
    pub child_object_ids: Vec<U32Id<BMeshObject>>,
}
