use crate::ext::QbclExtNodeBody;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per scene-node provenance in the `qbcl` ext, keyed by hierarchy node id.
/// The name derives from the hierarchy node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbclExtNode {
    /// Whether the node is shown in the editor.
    pub visible: bool,

    /// Whether the node is locked in the editor.
    pub locked: bool,

    /// The per-kind body.
    pub body: QbclExtNodeBody,
}

impl Default for QbclExtNode {
    fn default() -> Self {
        Self {
            visible: true,
            locked: false,
            body: QbclExtNodeBody::default(),
        }
    }
}
