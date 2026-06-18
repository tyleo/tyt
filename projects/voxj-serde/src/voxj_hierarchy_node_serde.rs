use crate::VoxjTransformSerde;
use serde::{Deserialize, Serialize};
use voxj::VoxjHierarchyNode;

/// Serde-compatible parity type for [`VoxjHierarchyNode`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxjHierarchyNodeSerde {
    pub name: String,
    pub child_nodes: Vec<usize>,
    pub child_objects: Vec<usize>,
    pub transform: VoxjTransformSerde,
}

impl From<VoxjHierarchyNode> for VoxjHierarchyNodeSerde {
    fn from(v: VoxjHierarchyNode) -> Self {
        Self {
            name: v.name,
            child_nodes: v.child_nodes,
            child_objects: v.child_objects,
            transform: v.transform.into(),
        }
    }
}

impl From<VoxjHierarchyNodeSerde> for VoxjHierarchyNode {
    fn from(v: VoxjHierarchyNodeSerde) -> Self {
        Self {
            name: v.name,
            child_nodes: v.child_nodes,
            child_objects: v.child_objects,
            transform: v.transform.into(),
        }
    }
}
