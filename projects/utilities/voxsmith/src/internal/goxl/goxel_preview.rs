use serde::{Deserialize, Serialize};

/// The `PREV` preview thumbnail preserved in the `goxel` ext: the decoded
/// `RGBA` pixels Goxel renders for the file. It has no native voxcore home, so
/// it rides here so the chunk is written back unchanged.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GoxelPreview {
    /// Image width, in pixels.
    pub width: u32,

    /// Image height, in pixels.
    pub height: u32,

    /// `[r, g, b, a]` pixels in image order, `width * height` of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pixels: Vec<[u8; 4]>,
}
