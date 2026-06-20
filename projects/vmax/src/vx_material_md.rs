#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Extended material-dispersion parameters (`md`) Voxel Max stores on a material
/// slot: absorption (`a`), index of refraction (`i`), and transmission (`t`).
/// Present on materials authored in Voxel Max; absent from the slots `from-voxj`
/// rebuilds (voxj carries only metalness/roughness/emission/shadows).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXMaterialMd {
    /// Absorption (`a`).
    pub a: f64,
    /// Index of refraction (`i`).
    pub i: f64,
    /// Transmission (`t`).
    pub t: f64,
}
