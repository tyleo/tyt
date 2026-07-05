use serde::{Deserialize, Serialize};

/// A material's dispersion parameters, preserved in the `voxel-max` ext so a
/// material with dispersion round-trips exactly. A pool cannot hold the absent
/// case, so the presence of dispersion lives here rather than in the palette.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxMaterialDispersion {
    /// Absorption (Voxel Max `a`).
    pub absorption: f64,

    /// Index of refraction (Voxel Max `i`).
    pub ior: f64,

    /// Transmission through the surface (Voxel Max `t`).
    pub transmission: f64,
}
