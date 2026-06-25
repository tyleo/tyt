use crate::{MVoxGroupNode, MVoxShapeNode, MVoxTransformNode};

/// The per-kind body of a scene node, one variant per scene-graph chunk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MVoxSceneNodeBody {
    /// An `nTRN` transform node.
    Transform(MVoxTransformNode),

    /// An `nGRP` group node.
    Group(MVoxGroupNode),

    /// An `nSHP` shape node.
    Shape(MVoxShapeNode),
}
