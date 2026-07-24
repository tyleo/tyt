use crate::{VoxjArrayProperty, VoxjScalarProperty};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material palette: named properties bound to value pools, and the
/// materials that index them.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjPalette {
    /// Ordered per-material properties. Property order fixes the cell order
    /// within each [`materials`](Self::materials) row.
    pub array_properties: Vec<VoxjArrayProperty>,

    /// Palette-scoped properties, each pinned to a single pool cell, with no
    /// [`materials`](Self::materials) column.
    pub scalar_properties: Vec<VoxjScalarProperty>,

    /// Row-major materials: one row per material, each of exactly
    /// `array_properties.len()` value-indices in property order, so the
    /// material count `M` is `materials.len()`. `materials[m][b]` is a
    /// value-index into the pool bound by array property `b`.
    pub materials: Vec<Vec<usize>>,
}
