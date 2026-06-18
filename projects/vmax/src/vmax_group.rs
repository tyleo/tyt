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
}
