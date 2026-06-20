#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-object Voxel Max camera (`.vmaxb` `cam`): orbit/light angles, pan,
/// zoom, and orbit origin. Stored so `from-voxj` restores the saved viewpoint for
/// the object.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXCamera {
    /// Camera width angle (`wa`).
    pub wa: f64,
    /// Camera height angle (`ha`).
    pub ha: f64,
    /// Camera distance angle (`da`).
    pub da: f64,
    /// Light width angle (`lwa`).
    pub lwa: f64,
    /// Light height angle (`lha`).
    pub lha: f64,
    /// Light distance angle (`lda`).
    pub lda: f64,
    /// Pan x (`px`).
    pub px: f64,
    /// Pan y (`py`).
    pub py: f64,
    /// Zoom (`z`).
    pub z: f64,
    /// Orbit origin `[x, y, z]` (`o`).
    pub o: [f64; 3],
}
