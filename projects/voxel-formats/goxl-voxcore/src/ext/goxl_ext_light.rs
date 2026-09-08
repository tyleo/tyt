#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The `LIGH` light-and-shading settings preserved in the `goxl` ext. They
/// have no native voxcore home, so they ride here verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxlExtLight {
    /// Light pitch, in radians.
    pub pitch: f32,

    /// Light yaw, in radians.
    pub yaw: f32,

    /// Light intensity.
    pub intensity: f32,

    /// Whether the light direction is fixed relative to the camera.
    pub fixed: bool,

    /// Ambient light amount.
    pub ambient: f32,

    /// Shadow amount.
    pub shadow: f32,

    /// Any further light-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, Vec<u8>)>,
}
