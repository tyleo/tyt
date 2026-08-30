use crate::VMaxExtent;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-snapshot statistics: Morton-coded occupied/selection bounds and counts.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxStats {
    /// Occupied-range minimum corner; `min[3]` is the Morton code of the first
    /// `ds` slot and anchors voxel decoding.
    pub min: Vec<i64>,

    /// Occupied-range maximum corner.
    pub max: Vec<i64>,

    /// Snapshot [`VMaxExtent`](crate::VMaxExtent) (`{o: <order>}`).
    pub extent: VMaxExtent,

    /// Occupied voxel count.
    pub count: i64,

    /// Selection-range minimum corner.
    pub smin: Vec<i64>,

    /// Selection-range maximum corner.
    pub smax: Vec<i64>,

    /// Selected voxel count.
    pub scount: i64,
}
