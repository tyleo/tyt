/// An 8-bit RGBA pixel grid: the decoded form of the PNG a `BL16` voxel block
/// or the `PREV` preview stores. The PNG traits exchange it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoxlRgbaImage {
    /// Image width, in pixels.
    pub width: u32,

    /// Image height, in pixels.
    pub height: u32,

    /// `[r, g, b, a]` pixels in row-major image order, left to right then top
    /// to bottom, [`width`](Self::width)` * `[`height`](Self::height) of them.
    pub pixels: Vec<[u8; 4]>,
}
