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

    /// Whether the region is a flat work plane (`f`).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "f", default, skip_serializing_if = "Option::is_none")
    )]
    pub flat: Option<bool>,
}
