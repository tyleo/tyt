use crate::{BVoxHierarchyNode, BVoxObject};
use branded_id::U32Id;
use ty_math::TyTransformF64;

/// A hierarchy node: references child nodes and objects by branded id and
/// carries a transform. Nodes form a DAG.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxHierarchyNode {
    /// The display name of the node.
    pub name: String,

    /// The child nodes beneath this node.
    pub child_nodes: Vec<U32Id<BVoxHierarchyNode>>,

    /// The objects placed by this node.
    pub child_objects: Vec<U32Id<BVoxObject>>,

    /// The transform applied to this node and everything beneath it.
    pub transform: TyTransformF64,
}
