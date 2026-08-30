/// A decoded `.vmax` `QuickLook/` thumbnail, held as a pixel grid. Encoded to
/// and from PNG, not serde. The round-trip is pixel-lossless for the 8-bit
/// images Voxel Max writes; bytes are re-encoded, not preserved verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxImage {
    /// Image width in pixels.
    pub width: u32,

    /// Image height in pixels.
    pub height: u32,

    /// Row-major pixels, one `[r, g, b, a]` cell each; `width * height` long.
    pub pixels: Vec<[u8; 4]>,
}
