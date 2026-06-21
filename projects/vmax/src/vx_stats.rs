use crate::VXExtent;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-snapshot statistics: the Morton-coded occupied/selection bounds and
/// counts Voxel Max records for each snapshot. `min[3]` (the Morton code of the
/// snapshot's first `ds` slot) also anchors voxel decoding.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXStats {
    /// Occupied-range minimum corner; `min[3]` is the Morton code of the first
    /// `ds` slot.
    pub min: Vec<i64>,

    /// Occupied-range maximum corner.
    pub max: Vec<i64>,

    /// Snapshot [`VXExtent`](crate::VXExtent) (`{o: <order>}`).
    pub extent: VXExtent,

    /// Occupied voxel count.
    pub count: i64,

    /// Selection-range minimum corner.
    pub smin: Vec<i64>,

    /// Selection-range maximum corner.
    pub smax: Vec<i64>,

    /// Selected voxel count.
    pub scount: i64,
}
