/// An RGBA8 image of a baked material atlas: one `[r, g, b, a]` cell per
/// texel, row-major from the top-left.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AtlasImage {
    /// Width in texels.
    pub width: u32,

    /// Height in texels.
    pub height: u32,

    /// `width * height` cells.
    pub pixels: Vec<[u8; 4]>,
}
