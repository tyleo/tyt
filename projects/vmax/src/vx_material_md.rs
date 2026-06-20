#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Extended material-dispersion parameters Voxel Max stores on a material
/// slot: absorption, index of refraction, and transmission.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXMaterialMd {
    /// Absorption.
    pub a: f64,
    /// Index of refraction.
    pub i: f64,
    /// Transmission.
    pub t: f64,
}
