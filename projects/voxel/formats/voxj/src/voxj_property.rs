#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One per-material property of a [`VoxjPalette`](crate::VoxjPalette), bound
/// to a [`VoxjValuePool`](crate::VoxjValuePool).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjProperty {
    /// The property name. Advisory: a consumer ignores names it does not
    /// recognize. No property name repeats within a palette.
    pub name: String,

    /// Index into
    /// [`VoxjRuntimeState::value_pools`](crate::VoxjRuntimeState::value_pools);
    /// the value pool this property draws its values from.
    pub value_pool: usize,
}
