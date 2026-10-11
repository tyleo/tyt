#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Scene-level camera and key-light rig (`cam`). Distinct from a per-object
/// [`VMaxCamera`](crate::VMaxCamera).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxSceneCamera {
    /// Camera declination/pitch angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub da: f64,

    /// Camera azimuth/heading angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub ha: f64,

    /// Light declination/pitch angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub lda: f64,

    /// Light azimuth/heading angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub lha: f64,

    /// Light "world" angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub lwa: f64,

    /// Camera target/origin position.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub o: [f64; 3],

    /// Camera pan X.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub px: f64,

    /// Camera pan Y.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub py: f64,

    /// Camera "world" angle.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub wa: f64,

    /// Camera distance/zoom.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub z: f64,

    /// Authoritative camera orientation quaternion in current Voxel Max files.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub aq: Option<[f64; 4]>,

    /// Orthographic projection selection.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub op: Option<bool>,

    /// Camera zoom scale factor.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub zf: Option<f64>,
}
