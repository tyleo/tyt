use serde::{Deserialize, Serialize};
use vmax::VMaxObject;

/// Serde-compatible parity type for [`VMaxObject`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMaxObjectSerde {
    #[serde(rename = "n", default)]
    pub name: String,
    #[serde(default)]
    pub data: String,
    #[serde(rename = "pal", default)]
    pub palette: String,
    #[serde(rename = "hist", default)]
    pub history: String,
    #[serde(default)]
    pub id: String,
    #[serde(rename = "pid", skip_serializing_if = "Option::is_none", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "t_p")]
    pub position: [f64; 3],
    #[serde(rename = "t_r")]
    pub rotation: [f64; 4],
    #[serde(rename = "t_s")]
    pub scale: [f64; 3],
    #[serde(rename = "e_c", default)]
    pub center: [f64; 3],
}

impl From<VMaxObject> for VMaxObjectSerde {
    fn from(v: VMaxObject) -> Self {
        Self {
            name: v.name,
            data: v.data,
            palette: v.palette,
            history: v.history,
            id: v.id,
            parent_id: v.parent_id,
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
            center: v.center,
        }
    }
}

impl From<VMaxObjectSerde> for VMaxObject {
    fn from(v: VMaxObjectSerde) -> Self {
        Self {
            name: v.name,
            data: v.data,
            palette: v.palette,
            history: v.history,
            id: v.id,
            parent_id: v.parent_id,
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
            center: v.center,
        }
    }
}
