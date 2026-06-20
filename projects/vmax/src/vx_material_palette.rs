use crate::VXMaterial;
use serde::{Deserialize, Serialize};

/// Material palette decoded from a `palette*.settings.vmaxpsb` plist. Decode
/// reads `name`, `materials`, and the `colors` RGBA table; the remaining fields
/// are written so Voxel Max accepts a rebuilt palette.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXMaterialPalette {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub materials: Vec<VXMaterial>,
    /// Packed RGBA color table (4 bytes per entry). This is Voxel Max's color
    /// source when an object's `palette*.png` image is absent.
    #[serde(default, with = "serde_bytes")]
    pub colors: Vec<u8>,
    // Written for Voxel Max but never read back (voxj drops them).
    #[serde(skip_deserializing)]
    pub indices: Vec<i64>,
    #[serde(skip_deserializing, serialize_with = "serde_bytes::serialize")]
    pub lc: Vec<u8>,
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
