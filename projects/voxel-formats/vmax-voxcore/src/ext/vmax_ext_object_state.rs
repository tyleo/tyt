#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use vmax::{VMaxBrush, VMaxCamera, VMaxTools};

/// Per-object Voxel Max editor state preserved in the `vmax` ext, kept
/// aligned by index with the objects so a rebuilt object restores the state
/// Voxel Max needs to import it.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxExtObjectState {
    /// Object content UUID.
    pub uuid: String,

    /// Codable version.
    pub v: i64,

    /// Tool state.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tools: Option<VMaxTools>,

    /// Brush palette.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub brush: Option<VMaxBrush>,

    /// Per-object camera.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cam: Option<VMaxCamera>,
}
