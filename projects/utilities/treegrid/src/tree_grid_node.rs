use crate::{BTreeGridNode, TreeGridCellFormat, TreeGridLabel, TreeGridValue};
use branded_id::U32Id;

/// A node in a [`TreeGrid`](crate::TreeGrid).
///
/// A node with at least one value is a data node; a node may have both
/// values and children. Children attach at creation through
/// [`TreeGrid::add_child`](crate::TreeGrid::add_child) and are never
/// re-parented, so the forest cannot cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridNode {
    /// One path segment.
    pub label: TreeGridLabel,

    /// A verbatim suffix shown only by the `hierarchy` layout, joined
    /// to the label with one space; the caller supplies its own
    /// brackets (for example `(Group)`).
    pub annotation: Option<String>,

    /// How this node's values render to cells.
    pub format: TreeGridCellFormat,

    /// The node's data series.
    pub values: Vec<TreeGridValue>,

    // Crate-private so children attach only at creation, keeping the
    // forest single-parent and acyclic by construction.
    pub(crate) children: Vec<U32Id<BTreeGridNode>>,
}

impl TreeGridNode {
    pub(crate) fn new(label: TreeGridLabel) -> Self {
        Self {
            label,
            annotation: None,
            format: TreeGridCellFormat::default(),
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Child ids, in insertion order.
    pub fn children(&self) -> &[U32Id<BTreeGridNode>] {
        &self.children
    }
}
