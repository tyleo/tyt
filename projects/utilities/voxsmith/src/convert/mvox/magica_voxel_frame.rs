use serde::{Deserialize, Serialize};

/// One transform-node keyframe preserved in the `magica-voxel` ext. The voxcore
/// transform projects only the first frame, and as a quaternion that cannot be
/// inverted back to the packed rotation byte exactly, so the exact frame data
/// is kept here.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelFrame {
    /// `_r`: the packed signed-permutation rotation byte.
    pub rotation: u8,

    /// `_t`: the `[x, y, z]` integer translation.
    pub translation: [i32; 3],

    /// `_f`: the frame index this keyframe applies to, counting from `0`.
    #[serde(
        rename = "frame-index",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_index: Option<u32>,

    /// Any further frame-attribute keys, preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}
