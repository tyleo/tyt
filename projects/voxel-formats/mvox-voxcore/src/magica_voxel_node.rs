use crate::MagicaVoxelNodeBody;
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance preserved in the `magica-voxel` ext: the original
/// node id and the node-attributes `DICT`, plus the per-kind body. Aligned by
/// index with the hierarchy nodes, in scene-node order.
///
/// The voxcore node keeps a deduplicated structural view and a transform
/// projection; this holds the exact ids, attributes, references, and the frame
/// data the voxcore node cannot represent.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MagicaVoxelNode {
    /// The scene-node id other nodes reference (`id`).
    pub id: i32,

    /// `_name`: the node's display name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    /// `_hidden`: whether the node is hidden.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hidden: Option<bool>,

    /// Any further node-attribute keys, preserved verbatim.
    #[cfg_attr(
        feature = "ext",
        serde(rename = "attr-extra", default, skip_serializing_if = "Vec::is_empty")
    )]
    pub attr_extra: Vec<(String, String)>,

    /// The per-kind body.
    pub body: MagicaVoxelNodeBody,
}
