/// A decoded image from a `.vmax` package: a Voxel Max `QuickLook/` thumbnail,
/// held as its pixel grid rather than raw PNG bytes. [`pixels`](Self::pixels)
/// runs row-major, one `[r, g, b, a]` cell per pixel; the grid is
/// [`width`](Self::width) by [`height`](Self::height).
///
/// It is decoded from and re-encoded to PNG rather than through serde, so it
/// carries no serde derives. The round-trip is pixel-lossless for the 8-bit
/// images Voxel Max writes (the bytes are re-encoded, not preserved verbatim).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxImage {
    /// Image width in pixels.
    pub width: u32,

    /// Image height in pixels.
    pub height: u32,

    /// Row-major pixels, one `[r, g, b, a]` cell each; `width * height` long.
    pub pixels: Vec<[u8; 4]>,
}
