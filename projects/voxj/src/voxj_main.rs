use crate::{VoxjHierarchyNode, VoxjObject, VoxjPalette};

/// The body of a Voxel Json document: objects, palettes, and the hierarchy
/// that places them.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjMain {
    pub objects: Vec<VoxjObject>,
    pub palettes: Vec<VoxjPalette>,
    pub hierarchy_nodes: Vec<VoxjHierarchyNode>,
    /// Indices into `hierarchy_nodes`; the scene's roots.
    pub root_hierarchy_nodes: Vec<usize>,
}
