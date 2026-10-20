#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The seam a `cells` pattern fills between its cells.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjBorder {
    /// The material filling the seam.
    pub material: usize,

    /// How wide the seam runs.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub width: f64,
}
