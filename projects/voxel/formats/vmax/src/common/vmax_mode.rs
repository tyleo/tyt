#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Innermost leaf of a Voxel Max tool-mode entry: the mode-token dictionary.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxMode {
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
