use crate::MVoxFrame;

/// A transform node (`nTRN`): places its single child node in space across one
/// or more animation frames.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxTransformNode {
    /// The id of the child node this transform places.
    pub child: i32,

    /// The layer this node belongs to, or `-1` for none.
    pub layer: i32,

    /// The animation frames; a static node has exactly one.
    pub frames: Vec<MVoxFrame>,
}
