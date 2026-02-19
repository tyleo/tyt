use serde::{Deserialize, Serialize};
use vmax::VMaxGroup;

/// Serde-compatible parity type for [`VMaxGroup`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMaxGroupSerde {
    pub name: String,
    pub id: String,
    #[serde(rename = "pid", skip_serializing_if = "Option::is_none", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "t_p")]
    pub position: [f64; 3],
    #[serde(rename = "t_r")]
    pub rotation: [f64; 4],
    #[serde(rename = "t_s")]
    pub scale: [f64; 3],
}

impl From<VMaxGroup> for VMaxGroupSerde {
    fn from(v: VMaxGroup) -> Self {
        Self {
            name: v.name,
            id: v.id,
            parent_id: v.parent_id,
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}

impl From<VMaxGroupSerde> for VMaxGroup {
    fn from(v: VMaxGroupSerde) -> Self {
        Self {
            name: v.name,
            id: v.id,
            parent_id: v.parent_id,
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}
