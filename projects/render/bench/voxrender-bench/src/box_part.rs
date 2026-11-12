use crate::BoxFaces;

/// One run of a [`BoxDraw`](crate::BoxDraw)'s boxes and what draws them.
pub struct BoxPart<'a> {
    /// The fragment entry that shades the run's faces.
    pub fragment: &'a str,

    pub faces: BoxFaces,

    /// Which of the draw's shader modules holds the run's entries.
    pub module: usize,

    /// Each box's three `vec4<u32>`s, in draw order.
    pub boxes: Vec<[u32; 12]>,

    /// Whether the run draws glass. Its fragments write no depth. They
    /// write a light and the share of the target's color to keep under it
    /// as dual-source blending's two sources.
    pub glass: bool,
}
