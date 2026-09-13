/// A decoded image of a mesh document: one `[r, g, b, a]` cell per texel,
/// row-major from the top-left, the form the per-texel sampler reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DecodedImage {
    /// Width in texels.
    pub width: u32,

    /// Height in texels.
    pub height: u32,

    /// `width * height` cells.
    pub pixels: Vec<[u8; 4]>,
}
