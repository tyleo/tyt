#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use vmax::VMaxCamera;

/// Per-object Voxel Max state preserved in the `vmax` ext, so a rebuilt object
/// restores what Voxel Max needs to open it: its contents version and its
/// camera frame and supported workspace size. A synthesized object takes a
/// camera framed on its geometry and automatic workspace sizing.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxExtObjectState {
    /// Object content UUID.
    pub uuid: String,

    /// Codable version.
    pub v: i64,

    /// Per-object camera.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cam: Option<VMaxCamera>,

    /// The editor cube order captured from a loaded viewport, or its storage
    /// extent when no viewport is present.
    /// Absent for a synthesized object; automatic export then starts at 256.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub extent_order: Option<i64>,

    /// The live content center in the internal grid where `cam` was captured.
    /// Export moves the camera target with this center when changing grids.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub camera_reference_center: Option<[f64; 3]>,
}
