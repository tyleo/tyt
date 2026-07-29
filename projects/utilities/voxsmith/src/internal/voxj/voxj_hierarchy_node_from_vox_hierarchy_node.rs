use ty_math::TyTransformF64;
use voxcore::VoxHierarchyNode;
use voxj::{VoxjHierarchyNode, VoxjTransform};

/// Builds a [`VoxjHierarchyNode`] from a [`VoxHierarchyNode`], mapping branded
/// child ids back to indices and the transform back to its voxj form.
pub fn voxj_hierarchy_node_from_vox_hierarchy_node(node: &VoxHierarchyNode) -> VoxjHierarchyNode {
    VoxjHierarchyNode {
        name: node.name.clone(),
        child_nodes: node
            .child_node_ids
            .iter()
            .map(|id| id.to_u32() as usize)
            .collect(),
        child_objects: node
            .child_object_ids
            .iter()
            .map(|id| id.to_u32() as usize)
            .collect(),
        transform: voxj_transform_from_vox_transform(&node.transform),
    }
}

/// Converts a [`TyTransformF64`] into a [`VoxjTransform`].
fn voxj_transform_from_vox_transform(transform: &TyTransformF64) -> VoxjTransform {
    VoxjTransform {
        position: transform.position.to_array(),
        rotation: transform.rotation.to_array(),
        scale: transform.scale.to_array(),
    }
}
