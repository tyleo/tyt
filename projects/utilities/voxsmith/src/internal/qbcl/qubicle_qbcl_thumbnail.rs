use serde::{Deserialize, Serialize};

/// The preview thumbnail embedded in a `.qbcl` header, preserved in the
/// `qubicle-qbcl` ext as its decoded `RGBA` pixels. It has no native voxcore
/// home.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct QubicleQbclThumbnail {
    /// Image width, in pixels.
    pub width: u32,

    /// Image height, in pixels.
    pub height: u32,

    /// `[r, g, b, a]` pixels in row order, `width * height` of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pixels: Vec<[u8; 4]>,
}
