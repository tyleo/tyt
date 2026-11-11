/// One run of a [`BoxDraw`](crate::BoxDraw)'s boxes and the pipeline that
/// draws them.
pub struct BoxPart<'a> {
    /// The vertex entry that places the run's corners.
    pub vertex: &'a str,

    /// The fragment entry that shades the run's faces.
    pub fragment: &'a str,

    /// Whether the run draws its boxes' back faces in place of their front
    /// faces.
    pub backs: bool,

    /// Each box's two `vec4<u32>`s, in draw order.
    pub boxes: Vec<[u32; 8]>,
}
