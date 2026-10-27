#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The axes a 2D mirror reflects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SdfjAxes2d {
    /// `u`.
    U,

    /// `uv`.
    Uv,

    /// `v`.
    V,
}
