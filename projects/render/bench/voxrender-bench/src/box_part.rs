use crate::BoxFaces;

/// One part of a [`BoxDraw`](crate::BoxDraw): what draws the boxes of its
/// runs.
pub struct BoxPart<'a> {
    /// The fragment entry that shades the run's faces.
    pub fragment: &'a str,

    pub faces: BoxFaces,

    /// Which of the draw's shader modules holds the run's entries.
    pub module: usize,

    /// The boxes the part adds to the draw's room.
    pub room: usize,

    /// Whether the run draws glass. Its fragments write no depth. They
    /// write a light and the share of the target's color to keep under it
    /// as dual-source blending's two sources.
    pub glass: bool,
}
