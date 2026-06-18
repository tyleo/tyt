use crate::VoxjTransform;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A hierarchy node: references child nodes and objects by index and carries a
/// transform. Nodes form a DAG (a node may have multiple parents; no cycles).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VoxjHierarchyNode {
    pub name: String,
    /// Indices into `VoxjMain::hierarchy_nodes`.
    pub child_nodes: Vec<usize>,
    /// Indices into `VoxjMain::objects`.
    pub child_objects: Vec<usize>,
    pub transform: VoxjTransform,
}
