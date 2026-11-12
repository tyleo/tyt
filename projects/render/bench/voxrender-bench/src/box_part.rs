use crate::BoxFaces;

/// One run of a [`BoxDraw`](crate::BoxDraw)'s boxes and what draws them.
pub struct BoxPart<'a> {
    /// The fragment entry that shades the run's faces.
    pub fragment: &'a str,

    pub faces: BoxFaces,

    /// Each box's three `vec4<u32>`s, in draw order.
    pub boxes: Vec<[u32; 12]>,
}
