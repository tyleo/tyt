use crate::MVoxExtNodeBody;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance preserved in the `mvox` ext, keyed by the
/// hierarchy node it belongs to. The name and child links come from the
/// voxcore node at write. This holds only what the scene cannot derive.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MVoxExtNode {
    /// The scene-node id other nodes reference (`id`).
    pub id: i32,

    /// `_hidden`: whether the node is hidden.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hidden: Option<bool>,

    /// Any further node-attribute keys, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "attr-extra", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub attr_extra: Vec<(String, String)>,

    /// The per-kind body.
    pub body: MVoxExtNodeBody,
}
