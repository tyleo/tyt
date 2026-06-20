#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The scene-level camera / light-rig state (`scene.json` `cam`) — the viewport
/// camera and key-light angles Voxel Max records for the whole scene. Distinct
/// from a per-object [`VXCamera`](crate::VXCamera). All values are preserved
/// verbatim; sub-keys are written together so they are modeled as plain fields.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxSceneCamera {
    /// Camera declination/pitch angle (`da`).
    pub da: f64,
    /// Camera azimuth/heading angle (`ha`).
    pub ha: f64,
    /// Light declination/pitch angle (`lda`).
    pub lda: f64,
    /// Light azimuth/heading angle (`lha`).
    pub lha: f64,
    /// Light "world" angle (`lwa`).
    pub lwa: f64,
    /// Camera target/origin position (`o`).
    pub o: [f64; 3],
    /// Camera pan X (`px`).
    pub px: f64,
    /// Camera pan Y (`py`).
    pub py: f64,
    /// Camera "world" angle (`wa`).
    pub wa: f64,
    /// Camera distance/zoom (`z`).
    pub z: f64,
}
