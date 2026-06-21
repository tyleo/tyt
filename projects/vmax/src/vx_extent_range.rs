#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The occupied-voxel range within a snapshot's extent. Voxel Max records the
/// chunk-local min/max corner of the snapshot's occupied voxels here, alongside
/// the order tag `o`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXExtentRange {
    /// Occupied-range minimum corner (chunk-local).
    pub min: Vec<i64>,

    /// Occupied-range maximum corner (chunk-local).
    pub max: Vec<i64>,
}
