use crate::{BTreeGridNode, TreeGrid, TreeGridCells};
use branded_id::U32Id;

impl<C: TreeGridCells> TreeGrid<C> {
    /// Data nodes in pre-order, each paired with its full dot-joined
    /// path.
    pub(crate) fn data_paths(&self) -> Vec<(String, U32Id<BTreeGridNode>)> {
        let mut paths = Vec::new();
        for &root in self.roots() {
            self.collect_data_paths(root, "", &mut paths);
        }
        paths
    }

    fn collect_data_paths(
        &self,
        id: U32Id<BTreeGridNode>,
        prefix: &str,
        paths: &mut Vec<(String, U32Id<BTreeGridNode>)>,
    ) {
        let node = self.node(id);
        let path = if prefix.is_empty() {
            node.label.render()
        } else {
            format!("{prefix}.{}", node.label.render())
        };
        if !node.values.is_empty() {
            paths.push((path.clone(), id));
        }
        for &child in node.children() {
            self.collect_data_paths(child, &path, paths);
        }
    }
}
