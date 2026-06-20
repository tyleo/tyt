use crate::VXExtent;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-snapshot statistics (`s.st`). On decode only `min[3]` (the Morton code of
/// the snapshot's first `ds` slot) is read; the rest is written for Voxel Max.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXStats {
    #[cfg_attr(feature = "serde", serde(default))]
    pub min: Vec<i64>,
    // The remaining stats are written for Voxel Max but never read back.
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub max: Vec<i64>,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub extent: VXExtent,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub count: i64,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub smin: Vec<i64>,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub smax: Vec<i64>,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub scount: i64,
}
