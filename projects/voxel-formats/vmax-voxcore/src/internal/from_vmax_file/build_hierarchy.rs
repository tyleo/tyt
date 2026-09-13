use crate::group_transform;
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::TyTransformF64;
use vmax::VMaxSceneJsonFile;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode};

/// Builds the voxcore hierarchy: one node per group then one per object, the
/// latter placing its geometry. `object_ids[i]` is the object that scene object
/// `i` places, so instances share a `child_objects` id. Returns the nodes in id
/// order and the root ids.
pub(crate) fn build_hierarchy(
    scene: &VMaxSceneJsonFile,
    object_transforms: &[TyTransformF64],
    object_ids: &[usize],
) -> (Vec<VoxHierarchyNode>, Vec<U32Id<BVoxHierarchyNode>>) {
    let mut nodes: Vec<VoxHierarchyNode> = Vec::new();
    let mut node_index_of_id: HashMap<&str, usize> = HashMap::new();
    let mut parents: Vec<Option<&str>> = Vec::new();

    for group in &scene.groups {
        node_index_of_id.insert(&group.id, nodes.len());
        parents.push(group.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: group.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: Vec::new(),
            transform: group_transform(group),
        });
    }
    for (index, object) in scene.objects.iter().enumerate() {
        node_index_of_id.insert(&object.id, nodes.len());
        parents.push(object.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: object.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![U32Id::<BVoxObject>::from_u32(object_ids[index] as u32)],
            transform: object_transforms[index],
        });
    }

    let mut roots = Vec::new();
    for (node_index, parent) in parents.iter().enumerate() {
        match parent.and_then(|pid| node_index_of_id.get(pid)) {
            Some(&parent_node_index) => nodes[parent_node_index]
                .child_node_ids
                .push(U32Id::from_u32(node_index as u32)),
            None => roots.push(U32Id::from_u32(node_index as u32)),
        }
    }

    (nodes, roots)
}
