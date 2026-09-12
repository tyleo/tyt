use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::BVoxObject;

/// One model reference of a shape node preserved in the `mvox` ext, in
/// stored order. This is the full list, so a shape that draws the same object
/// on several frames round-trips even though the voxcore node lists each
/// placed object only once. The model index written is the object's listing
/// index.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MVoxExtShapeModel {
    /// The object this entry draws.
    pub object: U32Id<BVoxObject>,

    /// `_f`: the frame index this model is shown on, counting from `0`.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "frame-index",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub frame_index: Option<u32>,

    /// Any further model-attribute keys, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub extra: Vec<(String, String)>,
}
