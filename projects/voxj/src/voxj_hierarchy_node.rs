use crate::VoxjTransform;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A hierarchy node: references child nodes and objects by index and carries a
/// transform. Nodes form a DAG.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjHierarchyNode {
    /// Display name of the node.
    pub name: String,
    /// Indices into [`VoxjMain::hierarchy_nodes`](crate::VoxjMain::hierarchy_nodes).
    pub child_nodes: Vec<usize>,
    /// Indices into [`VoxjMain::objects`](crate::VoxjMain::objects).
    pub child_objects: Vec<usize>,
    /// Transform applied to this node and everything beneath it.
    pub transform: VoxjTransform,
}
