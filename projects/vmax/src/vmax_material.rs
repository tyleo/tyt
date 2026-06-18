#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max material slot (one of the eight selectable per palette).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxMaterial {
    /// Metalness in `0..=1` (Voxel Max `mc`).
    pub metalness: f64,
    /// Roughness in `0..=1` (Voxel Max `rc`).
    pub roughness: f64,
    /// Emission strength (Voxel Max `sic`).
    pub emission: f64,
}
