#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One attribute-to-pool binding in a [`VoxjPalette`](crate::VoxjPalette),
/// fixing one column of its column-major materials.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjPaletteBinding {
    /// The bound attribute name. Advisory: a consumer ignores names it does not
    /// recognize. No two bindings in a palette share an attribute.
    pub attribute: String,

    /// Index into
    /// [`VoxjRuntimeState::value_pools`](crate::VoxjRuntimeState::value_pools);
    /// the pool this attribute draws its values from.
    pub pool_ref: usize,
}
