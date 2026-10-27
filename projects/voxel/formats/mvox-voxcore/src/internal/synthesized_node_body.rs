use crate::{
    MVoxExtFrame, MVoxExtNodeBody, SceneNodeKind, frame_translation, synthesized_shape_model,
};
use mvox::MVoxRotation;
use voxcore::VoxHierarchyNode;

/// The body a synthesized scene node of `kind` takes for `node`. A transform's
/// one frame drops any rotation or scale on the node.
pub fn synthesized_node_body(kind: SceneNodeKind, node: &VoxHierarchyNode) -> MVoxExtNodeBody {
    match kind {
        SceneNodeKind::Transform => MVoxExtNodeBody::Transform {
            layer: -1,
            frames: vec![MVoxExtFrame {
                rotation: MVoxRotation::IDENTITY.0,
                translation: frame_translation(node.transform.position),
                frame_index: None,
                extra: Vec::new(),
            }],
        },

        SceneNodeKind::Group => MVoxExtNodeBody::Group,

        SceneNodeKind::Shape => MVoxExtNodeBody::Shape {
            models: node
                .child_object_ids
                .iter()
                .map(|&object| synthesized_shape_model(object))
                .collect(),
        },
    }
}
