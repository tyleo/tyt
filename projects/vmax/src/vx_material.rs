#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single material slot decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXMaterial {
    /// Material slot index as a string `"1"`..`"8"` (written only).
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub mi: String,
    /// Metalness coefficient.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mc: f64,
    /// Roughness coefficient.
    #[cfg_attr(feature = "serde", serde(default))]
    pub rc: f64,
    /// Self-illumination (emission) coefficient.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sic: f64,
    /// Whether the material casts shadows.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sh: bool,
}
