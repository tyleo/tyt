use crate::{BTreeGridNode, TreeGridCellFormat, TreeGridLabel};
use branded_id::U32Id;

/// A node in a [`TreeGrid`](crate::TreeGrid), over the grid's value
/// type `V`.
///
/// A node with at least one value is a data node; a node may have both
/// values and children. Children attach at creation through
/// [`TreeGrid::add_child`](crate::TreeGrid::add_child) and are never
/// re-parented, so the forest cannot cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridNode<V> {
    /// One path segment.
    pub label: TreeGridLabel,

    /// A verbatim suffix joined with one space to the label wherever
    /// a layout labels this node; the caller supplies its own
    /// brackets (for example `(Group)`).
    pub annotation: Option<String>,

    /// How this node's values render to cells; unset defers to the
    /// grid's cell policy per value.
    pub format: Option<TreeGridCellFormat>,

    /// The node's data series.
    pub values: Vec<V>,

    // Crate-private so children attach only at creation, keeping the
    // forest single-parent and acyclic by construction.
    pub(crate) children: Vec<U32Id<BTreeGridNode>>,
}

impl<V> TreeGridNode<V> {
    pub(crate) fn new(label: TreeGridLabel) -> Self {
        Self {
            label,
            annotation: None,
            format: None,
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Child ids, in insertion order.
    pub fn children(&self) -> &[U32Id<BTreeGridNode>] {
        &self.children
    }
}
