#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One palette-scoped property of a [`VoxjPalette`](crate::VoxjPalette),
/// pinned to a single pool cell: one value for the whole palette, with no
/// per-material column.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjScalarProperty {
    /// The property name. Advisory: a consumer ignores names it does not
    /// recognize. No property name repeats within a palette, across its array
    /// and scalar properties together.
    pub name: String,

    /// Index into
    /// [`VoxjRuntimeState::value_pools`](crate::VoxjRuntimeState::value_pools);
    /// the pool the pinned value lives in.
    pub value_pool: usize,

    /// Index into the pool's values; the single value this property takes.
    pub value_index: usize,
}
