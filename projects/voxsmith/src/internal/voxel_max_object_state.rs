use serde::{Deserialize, Serialize};
use vmax::{VMaxBrush, VMaxCamera, VMaxTools};

/// Per-object Voxel Max editor state preserved in the `voxel-max` ext, kept
/// aligned by index with the objects so a rebuilt object restores the state
/// Voxel Max needs to import it.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct VoxelMaxObjectState {
    /// Object content UUID.
    pub uuid: String,

    /// Codable version.
    pub v: i64,

    /// Tool state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<VMaxTools>,

    /// Brush palette.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brush: Option<VMaxBrush>,

    /// Per-object camera.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cam: Option<VMaxCamera>,
}
