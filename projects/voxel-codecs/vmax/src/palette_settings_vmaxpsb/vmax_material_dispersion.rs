#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Dispersion parameters Voxel Max stores on a material slot (the optional `md`
/// block): absorption, index of refraction, and transmission.
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

    /// Light transmission through the surface.
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    pub transmission: f64,
}
