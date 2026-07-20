use crate::BTreeGridNode;
use branded_id::U32Id;

/// One group in the grouping walk: a branch and its direct data
/// children.
pub(crate) struct Group {
    /// The branch the heading names; `None` for the root-level
    /// group.
    pub(crate) branch: Option<U32Id<BTreeGridNode>>,

    /// The branch's depth in the forest, `0` at a root.
    pub(crate) depth: usize,

    /// The group's data nodes, in insertion order.
    pub(crate) members: Vec<U32Id<BTreeGridNode>>,
}
