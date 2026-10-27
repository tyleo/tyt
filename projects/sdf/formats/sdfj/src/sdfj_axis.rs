#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An axis: `x`, `y`, or `z`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SdfjAxis {
    /// `x`.
    X,

    /// `y`.
    Y,

    /// `z`.
    Z,
}
