use serde::{Deserialize, Serialize};

/// A single material slot decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXMaterial {
    /// Material slot index as a string `"1"`..`"8"` (written only).
    #[serde(skip_deserializing)]
    pub mi: String,
    /// Metalness coefficient.
    #[serde(default)]
    pub mc: f64,
    /// Roughness coefficient.
    #[serde(default)]
    pub rc: f64,
    /// Self-illumination (emission) coefficient.
    #[serde(default)]
    pub sic: f64,
    /// Whether the material casts shadows.
    #[serde(default)]
    pub sh: bool,
}
