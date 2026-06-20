use crate::VXMaterialMd;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single material slot decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXMaterial {
    /// Material slot index as a string `"1"`..`"8"`.
    pub mi: String,
    /// Metalness coefficient.
    pub mc: f64,
    /// Roughness coefficient.
    pub rc: f64,
    /// Self-illumination (emission) coefficient.
    pub sic: f64,
    /// Whether the material casts shadows.
    pub sh: bool,
    /// Extended dispersion parameters (`md`); present on Voxel Max-authored
    /// materials, absent from slots `from-voxj` rebuilds.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub md: Option<VXMaterialMd>,
}
