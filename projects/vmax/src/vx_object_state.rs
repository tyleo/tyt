use crate::{VXBrush, VXCamera, VXTools};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-object Voxel Max editor state preserved in the `voxel-max` ext so
/// `from-voxj` can rebuild the `.vmaxb` top level Voxel Max requires to import an
/// object: the content `uuid`/version plus the `tools`/`brush`/`cam` state. The
/// geometry lives in the voxj document, so the voxel `snapshots` are regenerated
/// on reconstruction rather than stored here.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXObjectState {
    /// Object content UUID (`uuid`).
    pub uuid: String,
    /// Codable version (`v`).
    pub v: i64,
    /// Tool state (`tools`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tools: Option<VXTools>,
    /// Brush palette (`brush`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub brush: Option<VXBrush>,
    /// Per-object camera (`cam`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cam: Option<VXCamera>,
}
