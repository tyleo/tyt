use crate::{BTreeGridNode, TreeGrid, TreeGridCells};
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

    /// The grouping walk behind `header` labels and nested tables:
    /// the data roots first, then one group per proper ancestor of a
    /// data node, depth-first, parents before children. Empty groups
    /// stay in: their headings mark the path to deeper data.
    pub(crate) fn groups(&self) -> Vec<Group> {
        let mut groups = Vec::new();
        let data_roots: Vec<U32Id<BTreeGridNode>> = self
            .roots()
            .iter()
            .copied()
            .filter(|&root| !self.node(root).values.is_empty())
            .collect();
        if !data_roots.is_empty() {
            groups.push(Group {
                branch: None,
                depth: 0,
                members: data_roots,
            });
        }
        for &root in self.roots() {
            self.collect_groups(root, 0, &mut groups);
        }
        groups
    }

    fn collect_groups(&self, id: U32Id<BTreeGridNode>, depth: usize, groups: &mut Vec<Group>) {
        if !self.leads_to_data(id) {
            return;
        }
        let node = self.node(id);
        groups.push(Group {
            branch: Some(id),
            depth,
            members: node
                .children()
                .iter()
                .copied()
                .filter(|&child| !self.node(child).values.is_empty())
                .collect(),
        });
        for &child in node.children() {
            self.collect_groups(child, depth + 1, groups);
        }
    }

    /// Whether `id` is a proper ancestor of a data node.
    fn leads_to_data(&self, id: U32Id<BTreeGridNode>) -> bool {
        self.node(id)
            .children()
            .iter()
            .any(|&child| !self.node(child).values.is_empty() || self.leads_to_data(child))
    }
}
