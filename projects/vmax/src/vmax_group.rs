#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A group node in a Voxel Max scene hierarchy.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxGroup {
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: String,
    #[cfg_attr(
        feature = "serde",
        serde(rename = "pid", skip_serializing_if = "Option::is_none", default)
    )]
    pub parent_id: Option<String>,
    #[cfg_attr(feature = "serde", serde(rename = "t_p"))]
    pub position: [f64; 3],
    #[cfg_attr(feature = "serde", serde(rename = "t_r"))]
    pub rotation: [f64; 4],
    #[cfg_attr(feature = "serde", serde(rename = "t_s"))]
    pub scale: [f64; 3],
    /// Hierarchy sort/path triple.
    #[cfg_attr(feature = "serde", serde(default))]
    pub ind: [i64; 3],
    /// Selection/visibility flag; present on some nodes.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub s: Option<bool>,
    /// Transform-anchor token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_al: String,
    /// Transform pivot-axis token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_pa: String,
    /// Transform pivot-face token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_pf: String,
    /// Center of the group's voxel bounds in model space.
    #[cfg_attr(feature = "serde", serde(rename = "e_c", default))]
    pub center: [f64; 3],
    /// Min corner of the group's voxel bounds, relative to
    /// [`center`](Self::center), when present.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "e_mi", skip_serializing_if = "Option::is_none", default)
    )]
    pub bounds_min: Option<[f64; 3]>,
    /// Max corner of the group's voxel bounds, relative to
    /// [`center`](Self::center), when present.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "e_ma", skip_serializing_if = "Option::is_none", default)
    )]
    pub bounds_max: Option<[f64; 3]>,
}
