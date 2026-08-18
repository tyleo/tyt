use crate::VoxjProperty;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material palette.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjPalette {
    /// Ordered per-material properties. The order of these properties fixes
    /// the cell order within each [`materials`](Self::materials) row.
    pub properties: Vec<VoxjProperty>,

    /// Row-major materials: one row per material, each of exactly
    /// `properties.len()` value-indices in property order, so the material
    /// count `M` is `materials.len()`. `materials[m][b]` is a
    /// value-index into the value pool bound by property `b`.
    pub materials: Vec<Vec<usize>>,
}
