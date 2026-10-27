use crate::{MVoxExtNode, SceneNodeKind, synthesized_node_body};
use branded_id::U32Id;
use std::collections::BTreeMap;
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode};

/// Inserts the entry a synthesized scene node of `kind` takes for hierarchy
/// node `node_id`, which is `node`, into `entries`. The scene-node id is one
/// past the largest in `entries`, counting up in insertion order. The
/// synthesizer and the retain hook both build entries here, which keeps a node
/// retained after the load equal to what synthesis would give it.
pub fn insert_synthesized_scene_node(
    entries: &mut BTreeMap<U32Id<BVoxHierarchyNode>, MVoxExtNode>,
    node_id: U32Id<BVoxHierarchyNode>,
    kind: SceneNodeKind,
    node: &VoxHierarchyNode,
) {
    let id = entries
        .values()
        .map(|entry| entry.id + 1)
        .max()
        .unwrap_or(0);
    entries.insert(
        node_id,
        MVoxExtNode {
            id,
            hidden: None,
            attr_extra: Vec::new(),
            body: synthesized_node_body(kind, node),
        },
    );
}
