use crate::VMaxExtent;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-snapshot statistics: Morton-coded occupied/selection bounds and counts,
/// plus the position-sum caches current Voxel Max derives from the voxels.
/// The snapshot's voxel bytes remain authoritative. An unknown key is an error.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxStats {
    /// Occupied-range minimum corner; `min[3]` is the Morton code of the first
    /// `ds` slot and anchors voxel decoding.
    pub min: Vec<i64>,

    /// Occupied-range maximum corner.
    pub max: Vec<i64>,

    /// Snapshot [`VMaxExtent`] (`{o: <order>}`).
    pub extent: VMaxExtent,

    /// Occupied voxel count.
    pub count: i64,

    /// Selection-range minimum corner.
    pub smin: Vec<i64>,

    /// Selection-range maximum corner.
    pub smax: Vec<i64>,

    /// Selected voxel count.
    pub scount: i64,

    /// Sum of the occupied voxels' positions, a cache current Voxel Max
    /// writes and opens without.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub psum: Option<Vec<i64>>,

    /// Sum of the selected voxels' positions, a cache like
    /// [`psum`](Self::psum).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub spsum: Option<Vec<i64>>,

    /// A value current Voxel Max writes beside the position-sum caches,
    /// meaning unknown.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub psv: Option<i64>,
}
