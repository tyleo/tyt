use crate::{BVoxHierarchyNode, BVoxObject};
use branded_id::U32Id;
use ty_math::TyTransformF64;

/// A node in the scene hierarchy: it references child nodes and child objects
/// by id and carries a transform applied to them.
///
/// Nodes form a DAG: a node may have several parents, so the same node can be
/// reused across the scene. Within one node, though, each direct child node and
/// child object appears at most once. The ids reference a
/// [`VoxMain`](crate::VoxMain) and are meaningful only within it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxHierarchyNode {
    /// Display name.
    pub name: String,

    /// Transform applied to this node and its subtree.
    pub transform: TyTransformF64,

    /// Child nodes.
    pub child_node_ids: Vec<U32Id<BVoxHierarchyNode>>,

    /// Objects placed by this node.
    pub child_object_ids: Vec<U32Id<BVoxObject>>,
}
