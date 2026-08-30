use crate::{MVoxNodeAttributes, MVoxSceneNodeBody};

/// One node of the scene graph: its id, shared attributes, and per-kind body.
/// Nodes reference one another by [`id`](Self::id); the graph is conventionally
/// rooted at the transform node with id `0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MVoxSceneNode {
    /// The node id other nodes reference.
    pub id: i32,

    /// The node-attributes `DICT` (`_name`, `_hidden`, ...).
    pub attributes: MVoxNodeAttributes,

    /// The per-kind body.
    pub body: MVoxSceneNodeBody,
}
