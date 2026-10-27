use crate::{BTreeGridNode, TreeGrid, TreeGridCells};
use branded_id::U32Id;

impl<C: TreeGridCells> TreeGrid<C> {
    /// Whether `id` is a proper ancestor of a data node.
    pub(crate) fn leads_to_data(&self, id: U32Id<BTreeGridNode>) -> bool {
        self.node(id)
            .children()
            .iter()
            .any(|&child| !self.node(child).values.is_empty() || self.leads_to_data(child))
    }
}
