#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Dispersion parameters on a material slot (the `md` block).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxMaterialDispersion {
    /// Absorption.
    #[cfg_attr(feature = "serde", serde(rename = "a"))]
    pub absorption: f64,

    /// Index of refraction.
    #[cfg_attr(feature = "serde", serde(rename = "i"))]
    pub ior: f64,

    /// Transmission through the surface.
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    pub transmission: f64,
}
