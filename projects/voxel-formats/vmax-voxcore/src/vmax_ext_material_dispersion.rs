#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A material's dispersion parameters, preserved in the `vmax` ext so a
/// material with dispersion round-trips exactly. A value pool cannot hold the
/// absent case, so the presence of dispersion lives here rather than in the
/// palette.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct VMaxExtMaterialDispersion {
    /// Absorption (Voxel Max `a`).
    pub absorption: f64,

    /// Index of refraction (Voxel Max `i`).
    pub ior: f64,

    /// Transmission through the surface (Voxel Max `t`).
    pub transmission: f64,
}
