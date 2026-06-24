use branded_id::U32Id;
use ty_math::{TyQuaternion, TyTransformF64, TyVector3};
use voxcore::VoxHierarchyNode;
use voxj::{VoxjHierarchyNode, VoxjTransform};

/// Builds a [`VoxHierarchyNode`] from a [`VoxjHierarchyNode`], mapping the child
/// node and object indices to branded ids (each index equals its id) and the
/// transform to its [`ty_math`] form.
pub(crate) fn vox_hierarchy_node_from_voxj_hierarchy_node(
    node: &VoxjHierarchyNode,
) -> VoxHierarchyNode {
    VoxHierarchyNode {
        name: node.name.clone(),
        child_nodes: node
            .child_nodes
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
        child_objects: node
            .child_objects
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
        transform: vox_transform_from_voxj_transform(&node.transform),
    }
}

/// Converts a [`VoxjTransform`] into a [`TyTransformF64`].
fn vox_transform_from_voxj_transform(transform: &VoxjTransform) -> TyTransformF64 {
    let [position_x, position_y, position_z] = transform.position;
    let [rotation_x, rotation_y, rotation_z, rotation_w] = transform.rotation;
    let [scale_x, scale_y, scale_z] = transform.scale;
    TyTransformF64::new(
        TyVector3::new(position_x, position_y, position_z),
        TyQuaternion::new(rotation_x, rotation_y, rotation_z, rotation_w),
        TyVector3::new(scale_x, scale_y, scale_z),
    )
}
