use crate::VoxjTransform;

/// A hierarchy node: references child nodes and objects by index and carries a
/// transform. Nodes form a DAG (a node may have multiple parents; no cycles).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjHierarchyNode {
    pub name: String,
    /// Indices into `VoxjMain::hierarchy_nodes`.
    pub child_nodes: Vec<usize>,
    /// Indices into `VoxjMain::objects`.
    pub child_objects: Vec<usize>,
    pub transform: VoxjTransform,
}
