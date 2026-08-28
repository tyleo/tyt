use serde::{Deserialize, Serialize};

/// One model reference of a shape node preserved in the `magica-voxel` ext, in
/// stored order. This is the full list, so a shape that draws the same model on
/// several frames round-trips even though the voxcore node lists each placed
/// object only once.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelShapeModel {
    /// The index of the model this entry draws.
    pub model: u32,

    /// `_f`: the frame index this model is shown on, counting from `0`.
    #[serde(
        rename = "frame-index",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_index: Option<u32>,

    /// Any further model-attribute keys, preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}
