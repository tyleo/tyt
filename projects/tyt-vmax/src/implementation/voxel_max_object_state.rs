use serde::{Deserialize, Serialize};
use vmax::{VMaxBrush, VMaxCamera, VMaxTools};

/// A [`VMaxSerdeContentsVmaxbFile`](vmax::VMaxSerdeContentsVmaxbFile)'s editor state apart from its
/// voxel [`snapshots`](vmax::VMaxSerdeContentsVmaxbFile::snapshots): the content `uuid`/version plus
/// the [`tools`](Self::tools)/[`brush`](Self::brush)/[`cam`](Self::cam) state.
/// Stored in the `voxel-max` ext so a `.vmax` package can be rebuilt without
/// re-storing the voxel geometry.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VoxelMaxObjectState {
    /// Object content UUID.
    pub uuid: String,
    /// Codable version.
    pub v: i64,
    /// Tool state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<VMaxTools>,
    /// Brush palette.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brush: Option<VMaxBrush>,
    /// Per-object camera.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cam: Option<VMaxCamera>,
}
