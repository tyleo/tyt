#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A `MATE` material preserved in the `goxel` ext, in stored order. A layer's
/// material is named by index into this list. Materials have no native voxcore
/// home, so they ride here verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxelMaterial {
    /// Material name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub name: String,

    /// `[r, g, b, a]` linear base color.
    #[cfg_attr(feature = "ext", serde(rename = "base-color"))]
    pub base_color: [f32; 4],

    /// Metallic factor.
    pub metallic: f32,

    /// Roughness factor.
    pub roughness: f32,

    /// `[r, g, b]` emission color.
    pub emission: [f32; 3],

    /// Any further material-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, Vec<u8>)>,
}
