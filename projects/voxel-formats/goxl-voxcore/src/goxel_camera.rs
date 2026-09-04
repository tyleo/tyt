#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A `CAMR` camera preserved in the `goxel` ext, in stored order. Cameras have
/// no native voxcore home, so they ride here verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxelCamera {
    /// Camera name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub name: String,

    /// `dist`: rotation distance from the target.
    pub distance: f32,

    /// `ortho`: whether the projection is orthographic.
    pub orthographic: bool,

    /// `mat`: the `4 x 4` camera-to-world transform.
    pub transform: [[f32; 4]; 4],

    /// `active`: whether this is the file's active camera.
    pub active: bool,

    /// Any further camera-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, Vec<u8>)>,
}
