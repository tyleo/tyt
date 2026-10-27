/// When [`downsample_objects`](crate::operations::object::downsample_objects())
/// keeps a block live, by how many of its cells are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeepRule {
    /// Every cell is live.
    All,

    /// Any cell is live.
    Any,

    /// At least half of the cells are live.
    Majority,
}
