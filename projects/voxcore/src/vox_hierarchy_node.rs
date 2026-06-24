use crate::{BVoxHierarchyNode, BVoxObject};
use branded_id::U32Id;
use ty_math::TyTransformF64;

/// A node in the scene hierarchy: it references child nodes and child objects by
/// id and carries a transform applied to them.
///
/// Nodes form a DAG, not a tree: a node may have several parents and may list the
/// same child twice (instancing). The ids reference a
/// [`VoxState`](crate::VoxState) and are meaningful only within it;
/// [`VoxState::validate`](crate::VoxState::validate) checks them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxHierarchyNode {
    /// Display name.
    pub name: String,

    /// Child nodes.
    pub child_nodes: Vec<U32Id<BVoxHierarchyNode>>,

    /// Objects placed by this node.
    pub child_objects: Vec<U32Id<BVoxObject>>,

    /// Transform applied to this node and its subtree.
    pub transform: TyTransformF64,
}
