use crate::{VMaxBrush, VMaxCamera, VMaxSnapshot, VMaxTools};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel object payload of a `contents*.vmaxb` binary plist. Beyond the voxel
/// [`snapshots`](Self::snapshots), the top level carries the editor state Voxel
/// Max requires to open and import the object (the content
/// [`uuid`](Self::uuid)/version and
/// [`tools`](Self::tools)/[`brush`](Self::brush)/[`cam`](Self::cam)).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxContentsVmaxbFile {
    /// Per-chunk voxel snapshots: the baked geometry edit log.
    #[cfg_attr(feature = "serde", serde(default))]
    pub snapshots: Vec<VMaxSnapshot>,

    /// Object content UUID.
    #[cfg_attr(feature = "serde", serde(default))]
    pub uuid: String,

    /// Codable version.
    #[cfg_attr(feature = "serde", serde(default))]
    pub v: i64,

    /// Tool state; separate from the voxel geometry.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tools: Option<VMaxTools>,

    /// Brush palette.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub brush: Option<VMaxBrush>,

    /// Per-object camera.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cam: Option<VMaxCamera>,
}
