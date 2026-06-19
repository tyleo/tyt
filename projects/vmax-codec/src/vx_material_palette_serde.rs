use crate::VXMaterialSerde;
use serde::{Deserialize, Serialize};
use vmax::VMaxMaterialPalette;

/// Material palette decoded from a `palette*.settings.vmaxpsb` plist. Decode
/// reads only `name` + `materials`; the remaining fields are written so Voxel
/// Max accepts a rebuilt palette.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXMaterialPaletteSerde {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub materials: Vec<VXMaterialSerde>,
    // Written for Voxel Max but never read back (voxj drops them).
    #[serde(skip_deserializing)]
    pub indices: Vec<i64>,
    #[serde(skip_deserializing, serialize_with = "serde_bytes::serialize")]
    pub lc: Vec<u8>,
    #[serde(skip_deserializing, serialize_with = "serde_bytes::serialize")]
    pub colors: Vec<u8>,
    #[serde(rename = "type", skip_deserializing)]
    pub palette_type: i64,
    #[serde(skip_deserializing)]
    pub transparency: f64,
    #[serde(skip_deserializing)]
    pub r: i64,
    #[serde(skip_deserializing)]
    pub rt: String,
    #[serde(skip_deserializing)]
    pub cmt: String,
    #[serde(skip_deserializing)]
    pub current: i64,
    #[serde(skip_deserializing)]
    pub ali: String,
}

impl VXMaterialPaletteSerde {
    /// Returns the decoded palette as a core [`VMaxMaterialPalette`].
    pub fn palette(&self) -> VMaxMaterialPalette {
        VMaxMaterialPalette {
            name: self.name.clone(),
            materials: self.materials.iter().cloned().map(Into::into).collect(),
        }
    }
}
