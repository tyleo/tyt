use crate::VoxjTransform;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A node in the scene hierarchy. Nodes form a DAG.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjHierarchyNode {
    /// Display name of the node.
    pub name: String,

    /// Transform applied to this node and everything beneath it.
    pub transform: VoxjTransform,

    /// Indices into
    /// [`VoxjRuntimeState::hierarchy_nodes`](crate::VoxjRuntimeState::hierarchy_nodes).
    pub child_nodes: Vec<usize>,

    /// Indices into
    /// [`VoxjRuntimeState::objects`](crate::VoxjRuntimeState::objects).
    pub child_objects: Vec<usize>,
}
