#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-object Voxel Max camera: orbit/light angles, pan, zoom, and orbit
/// origin.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxCamera {
    /// Camera width angle.
    pub wa: f64,

    /// Camera height angle.
    pub ha: f64,

    /// Camera distance angle.
    pub da: f64,

    /// Light width angle.
    pub lwa: f64,

    /// Light height angle.
    pub lha: f64,

    /// Light distance angle.
    pub lda: f64,

    /// Pan x.
    pub px: f64,

    /// Pan y.
    pub py: f64,

    /// Zoom.
    pub z: f64,

    /// Orbit origin `[x, y, z]`.
    pub o: [f64; 3],
}
