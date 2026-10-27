use crate::{VoxjHierarchyNode, VoxjTransform};

/// A node with the given children and an identity transform.
pub fn node(child_nodes: Vec<usize>, child_objects: Vec<usize>) -> VoxjHierarchyNode {
    VoxjHierarchyNode {
        name: "n".to_owned(),
        child_nodes,
        child_objects,
        transform: identity(),
    }
}

/// The identity transform: zero translation, identity rotation, unit scale.
fn identity() -> VoxjTransform {
    VoxjTransform {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}
