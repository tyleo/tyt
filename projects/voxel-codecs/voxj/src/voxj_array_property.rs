#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One per-material property of a [`VoxjPalette`](crate::VoxjPalette), bound
/// to a whole pool: each material row supplies one value-index into it.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjArrayProperty {
    /// The property name. Advisory: a consumer ignores names it does not
    /// recognize. No property name repeats within a palette, across its array
    /// and scalar properties together.
    pub name: String,

    /// Index into
    /// [`VoxjRuntimeState::value_pools`](crate::VoxjRuntimeState::value_pools);
    /// the pool this property draws its values from.
    pub value_pool: usize,
}
