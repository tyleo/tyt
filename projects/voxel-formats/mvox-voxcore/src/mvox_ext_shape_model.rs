#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// One model reference of a shape node preserved in the `mvox` ext, in
/// stored order. This is the full list, so a shape that draws the same model on
/// several frames round-trips even though the voxcore node lists each placed
/// object only once.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MVoxExtShapeModel {
    /// The index of the model this entry draws.
    pub model: u32,

    /// `_f`: the frame index this model is shown on, counting from `0`.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "frame-index",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub frame_index: Option<u32>,

    /// Any further model-attribute keys, preserved verbatim.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, String)>,
}
