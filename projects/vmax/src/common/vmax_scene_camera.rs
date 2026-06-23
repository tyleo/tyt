#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The scene-level camera / light-rig state: the viewport camera and key-light
/// angles Voxel Max records for the whole scene. Distinct from a per-object
/// [`VMaxCamera`](crate::VMaxCamera).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxSceneCamera {
    /// Camera declination/pitch angle.
    pub da: f64,

    /// Camera azimuth/heading angle.
    pub ha: f64,

    /// Light declination/pitch angle.
    pub lda: f64,

    /// Light azimuth/heading angle.
    pub lha: f64,

    /// Light "world" angle.
    pub lwa: f64,

    /// Camera target/origin position.
    pub o: [f64; 3],

    /// Camera pan X.
    pub px: f64,

    /// Camera pan Y.
    pub py: f64,

    /// Camera "world" angle.
    pub wa: f64,

    /// Camera distance/zoom.
    pub z: f64,
}
