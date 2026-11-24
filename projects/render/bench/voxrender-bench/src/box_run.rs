use crate::BViewPage;
use branded_id::UsizeId;

/// Boxes of a [`BoxDraw`](crate::BoxDraw) that follow one another and draw
/// through one part over one page's bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoxRun {
    /// The run's part among the draw's.
    pub part: usize,

    /// The page whose bindings the run's boxes read.
    pub page: UsizeId<BViewPage>,

    /// How many boxes the run holds.
    pub count: u32,
}
