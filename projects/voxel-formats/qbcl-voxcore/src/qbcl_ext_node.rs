use crate::QbclExtNodeBody;
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance preserved in the `qbcl` ext: the node
/// name, the editor flags, and the per-kind body. Aligned by index with the
/// hierarchy nodes, so the scene tree rebuilds exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct QbclExtNode {
    /// Node name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub name: String,

    /// Whether the node is shown in the editor.
    pub visible: bool,

    /// Whether the node is locked in the editor.
    pub locked: bool,

    /// The per-kind body.
    pub body: QbclExtNodeBody,
}
