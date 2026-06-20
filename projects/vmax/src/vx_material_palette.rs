use crate::VXMaterial;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Material palette decoded from a `palette*.settings.vmaxpsb` plist. Decode
/// reads `name`, `materials`, and the `colors` RGBA table; the remaining fields
/// are written so Voxel Max accepts a rebuilt palette.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXMaterialPalette {
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub materials: Vec<VXMaterial>,
    /// Packed RGBA color table (4 bytes per entry). This is Voxel Max's color
    /// source when an object's `palette*.png` image is absent.
    #[cfg_attr(feature = "serde", serde(default, with = "serde_bytes"))]
    pub colors: Vec<u8>,
    // Written for Voxel Max but never read back (voxj drops them).
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub indices: Vec<i64>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_deserializing, serialize_with = "serde_bytes::serialize")
    )]
    pub lc: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(rename = "type", skip_deserializing))]
    pub palette_type: i64,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub transparency: f64,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub r: i64,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub rt: String,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub cmt: String,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub current: i64,
    #[cfg_attr(feature = "serde", serde(skip_deserializing))]
    pub ali: String,
}
