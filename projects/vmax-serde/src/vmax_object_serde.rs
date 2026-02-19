use serde::{Deserialize, Serialize};
use vmax::VMaxObject;

/// Serde-compatible parity type for [`VMaxObject`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMaxObjectSerde {
    #[serde(rename = "n")]
    pub name: String,
    pub data: String,
    #[serde(rename = "pal")]
    pub palette: String,
    #[serde(rename = "hist")]
    pub history: String,
    pub id: String,
    #[serde(rename = "t_p")]
    pub position: [f64; 3],
    #[serde(rename = "t_r")]
    pub rotation: [f64; 4],
    #[serde(rename = "t_s")]
    pub scale: [f64; 3],
}

impl From<VMaxObject> for VMaxObjectSerde {
    fn from(v: VMaxObject) -> Self {
        Self {
            name: v.name,
            data: v.data,
            palette: v.palette,
            history: v.history,
            id: v.id,
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
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
            position: v.position,
            rotation: v.rotation,
            scale: v.scale,
        }
    }
}
