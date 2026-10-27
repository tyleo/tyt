#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The axes a 3D mirror reflects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SdfjAxes3d {
    /// `x`.
    X,

    /// `xy`.
    Xy,

    /// `xyz`.
    Xyz,

    /// `xz`.
    Xz,

    /// `y`.
    Y,

    /// `yz`.
    Yz,

    /// `z`.
    Z,
}
