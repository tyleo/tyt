use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use voxcore::BVoxValuePoolValue;

/// Per-palette Voxel Max provenance preserved in the `vmax` ext, holding what
/// the value pools have no place for. A slot is a value id in the palette's
/// material pools.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExtPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,

    /// The slots Voxel Max stores with no dispersion (`md`). Each writes with
    /// none while its dispersion values are the absent ones.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "slots-without-dispersion",
            default,
            skip_serializing_if = "BTreeSet::is_empty"
        )
    )]
    pub slots_without_dispersion: BTreeSet<U32Id<BVoxValuePoolValue>>,

    /// Each slot's transmission color coefficient (Voxel Max `tc`). Only some
    /// materials carry one.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "transmission-colors",
            default,
            skip_serializing_if = "BTreeMap::is_empty"
        )
    )]
    pub transmission_colors: BTreeMap<U32Id<BVoxValuePoolValue>, f64>,
}
