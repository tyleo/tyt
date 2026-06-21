#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The innermost leaf of a Voxel Max tool-mode entry: the dictionary holding
/// the mode tokens themselves (`{mo}`, `{m}`, `{mf, mo}`, or `{t, m}` depending
/// on the tool). Each token is optional so one struct models every observed
/// shape.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXMode {
    /// Primary mode token.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mo: Option<String>,
    /// Secondary mode token.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub m: Option<String>,
    /// Mode flag token.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mf: Option<String>,
    /// Type token.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub t: Option<String>,
}
