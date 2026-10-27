use crate::VMaxExtWorkArea;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use vmax::VMaxCamera;

/// Per-object Voxel Max state the ext keeps for rebuilding the contents file.
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

    /// The work area when it hides voxels. Absent, the work area is the edit
    /// grid.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub work_area: Option<VMaxExtWorkArea>,
}
