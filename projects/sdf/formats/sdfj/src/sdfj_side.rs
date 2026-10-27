#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One direction along an axis: `-x`, `-y`, `-z`, `+x`, `+y`, or `+z`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum SdfjSide {
    /// `-x`.
    #[cfg_attr(feature = "serde", serde(rename = "-x"))]
    NegativeX,

    /// `-y`.
    #[cfg_attr(feature = "serde", serde(rename = "-y"))]
    NegativeY,

    /// `-z`.
    #[cfg_attr(feature = "serde", serde(rename = "-z"))]
    NegativeZ,

    /// `+x`.
    #[cfg_attr(feature = "serde", serde(rename = "+x"))]
    PositiveX,

    /// `+y`.
    #[cfg_attr(feature = "serde", serde(rename = "+y"))]
    PositiveY,

    /// `+z`.
    #[cfg_attr(feature = "serde", serde(rename = "+z"))]
    PositiveZ,
}
