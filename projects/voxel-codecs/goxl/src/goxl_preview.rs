/// The `PREV` chunk: a preview thumbnail Goxel renders for the file, decoded
/// from its PNG into `RGBA` pixels. Goxel writes it at `128 x 128` and ignores
/// it on load.
///
/// [`pixels`](Self::pixels) holds [`width`](Self::width)` * `[`height`](Self::height)
/// cells in image order (row-major: left to right, top to bottom);
/// `goxl-codec` does the PNG conversion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoxlPreview {
    /// Image width, in pixels.
    pub width: u32,

    /// Image height, in pixels.
    pub height: u32,

    /// `[r, g, b, a]` pixels in image order, [`width`](Self::width)` * `[`height`](Self::height)
    /// of them.
    pub pixels: Vec<[u8; 4]>,
}
