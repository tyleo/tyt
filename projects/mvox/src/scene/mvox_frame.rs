use crate::{MVoxDict, MVoxRotation};

/// One keyframe of a transform node: the `_r` rotation and `_t` translation that
/// place the node, plus the optional `_f` frame index for animation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxFrame {
    /// `_r`: the rotation, [`MVoxRotation::IDENTITY`] when absent.
    pub rotation: MVoxRotation,

    /// `_t`: the `[x, y, z]` integer translation, `[0, 0, 0]` when absent.
    pub translation: [i32; 3],

    /// `_f`: the frame index this keyframe applies to, counting from `0`.
    pub frame_index: Option<u32>,

    /// Any further frame-attribute keys, preserved verbatim.
    pub extra: MVoxDict,
}
