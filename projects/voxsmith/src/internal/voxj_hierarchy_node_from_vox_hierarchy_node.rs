use ty_math::TyTransformF64;
use voxcore::VoxHierarchyNode;
use voxj::{VoxjHierarchyNode, VoxjTransform};

/// Builds a [`VoxjHierarchyNode`] from a [`VoxHierarchyNode`], mapping branded
/// child ids back to indices and the transform back to its voxj form.
pub(crate) fn voxj_hierarchy_node_from_vox_hierarchy_node(
    node: &VoxHierarchyNode,
) -> VoxjHierarchyNode {
    VoxjHierarchyNode {
        name: node.name.clone(),
        child_nodes: node
            .child_nodes
            .iter()
            .map(|id| id.to_u32() as usize)
            .collect(),
        child_objects: node
            .child_objects
            .iter()
            .map(|id| id.to_u32() as usize)
            .collect(),
        transform: voxj_transform_from_vox_transform(&node.transform),
    }
}

/// Converts a [`TyTransformF64`] into a [`VoxjTransform`].
fn voxj_transform_from_vox_transform(transform: &TyTransformF64) -> VoxjTransform {
    VoxjTransform {
        position: [
            transform.position.x,
            transform.position.y,
            transform.position.z,
        ],
        rotation: [
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        ],
        scale: [transform.scale.x, transform.scale.y, transform.scale.z],
    }
}
