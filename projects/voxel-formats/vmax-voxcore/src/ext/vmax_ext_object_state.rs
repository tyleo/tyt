#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use vmax::VMaxCamera;

/// Per-object Voxel Max state preserved in the `vmax` ext, so a rebuilt object
/// restores what Voxel Max needs to open it: its contents version and its
/// camera frame. A synthesized object keeps no camera.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxExtObjectState {
    /// Object content UUID.
    pub uuid: String,

    /// Codable version.
    pub v: i64,

    /// Per-object camera. Absent, the write frames the object's content.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cam: Option<VMaxCamera>,

    /// The live content center in the internal grid where `cam` was captured.
    /// Export moves the camera target with this center when changing grids.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub camera_reference_center: Option<[f64; 3]>,
}
