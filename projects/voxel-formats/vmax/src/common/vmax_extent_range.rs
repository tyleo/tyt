#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Occupied-voxel range within a snapshot's extent (`r`), chunk-local.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxExtentRange {
    /// Minimum corner.
    pub min: Vec<i64>,

    /// Maximum corner.
    pub max: Vec<i64>,
}
