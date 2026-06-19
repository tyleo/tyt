use serde::{Deserialize, Serialize};

/// Per-snapshot statistics (`s.st`). On decode only `min[3]` (the Morton code of
/// the snapshot's first `ds` slot) is read; the rest is written for Voxel Max.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXStatsSerde {
    #[serde(default)]
    pub min: Vec<i64>,
    // The remaining stats are written for Voxel Max but never read back.
    #[serde(skip_deserializing)]
    pub max: Vec<i64>,
    #[serde(skip_deserializing)]
    pub extent: Vec<i64>,
    #[serde(skip_deserializing)]
    pub count: i64,
    #[serde(skip_deserializing)]
    pub smin: Vec<i64>,
    #[serde(skip_deserializing)]
    pub smax: Vec<i64>,
    #[serde(skip_deserializing)]
    pub scount: i64,
}
