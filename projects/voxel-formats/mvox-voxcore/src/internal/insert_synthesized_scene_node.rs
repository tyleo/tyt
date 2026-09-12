use crate::{MVoxExtFrame, MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel, SceneNodeKind};
use branded_id::U32Id;
use mvox::MVoxRotation;
use std::collections::BTreeMap;
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode};

/// Inserts the entry a synthesized scene node of `kind` takes for hierarchy
/// node `node_id`, which is `node`, into `entries`. The scene-node id is one
/// past the largest in `entries`, counting up in insertion order. A transform
/// takes one identity-rotation frame at the node's translation rounded to
/// whole voxels, which drops any rotation or scale on the node. A shape draws
/// each placed object on its first frame. The synthesizer and the retain hook
/// both build entries here, which keeps a node retained after the load equal
/// to what synthesis would give it.
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
    let body = match kind {
        SceneNodeKind::Transform => MVoxExtNodeBody::Transform {
            layer: -1,
            frames: vec![MVoxExtFrame {
                rotation: MVoxRotation::IDENTITY.0,
                translation: node.transform.position.round().as_ivec3().to_array(),
                frame_index: None,
                extra: Vec::new(),
            }],
        },
        SceneNodeKind::Group => MVoxExtNodeBody::Group,
        SceneNodeKind::Shape => MVoxExtNodeBody::Shape {
            models: node
                .child_object_ids
                .iter()
                .map(|&object| MVoxExtShapeModel {
                    object,
                    frame_index: Some(0),
                    extra: Vec::new(),
                })
                .collect(),
        },
    };
    entries.insert(
        node_id,
        MVoxExtNode {
            id,
            hidden: None,
            attr_extra: Vec::new(),
            body,
        },
    );
}
