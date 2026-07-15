use crate::{BTreeGridNode, TreeGridLabel, TreeGridNode, TreeGridValue};
use branded_id::{IdVec, U32Id};

/// An ordered forest of labeled, data-bearing nodes in an append-only
/// arena.
///
/// Populate with [`add_root`](Self::add_root) and
/// [`add_child`](Self::add_child), then attach data with
/// [`push_value`](Self::push_value) and edit nodes through
/// [`node_mut`](Self::node_mut). Nodes attach to their parent at
/// creation and are never removed or re-parented, so the forest cannot
/// cycle. Ids are dense indices into this grid and are meaningful only
/// within it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeGrid {
    /// Dense node storage; a node's id is its index.
    nodes: IdVec<BTreeGridNode, TreeGridNode>,

    /// Root ids, in insertion order.
    roots: Vec<U32Id<BTreeGridNode>>,
}

impl TreeGrid {
    /// Appends a root node and returns its id.
    pub fn add_root(&mut self, label: TreeGridLabel) -> U32Id<BTreeGridNode> {
        let id = self.push_node(label);
        self.roots.push(id);
        id
    }

    /// Appends a child under `parent` and returns its id.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is not an id of this grid.
    pub fn add_child(
        &mut self,
        parent: U32Id<BTreeGridNode>,
        label: TreeGridLabel,
    ) -> U32Id<BTreeGridNode> {
        let id = self.push_node(label);
        self.nodes[parent.to_usize_id()].children.push(id);
        id
    }

    /// The node behind `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an id of this grid.
    pub fn node(&self, id: U32Id<BTreeGridNode>) -> &TreeGridNode {
        &self.nodes[id.to_usize_id()]
    }

    /// The node behind `id`, mutably. Children are not reachable
    /// through it; they attach only at creation.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an id of this grid.
    pub fn node_mut(&mut self, id: U32Id<BTreeGridNode>) -> &mut TreeGridNode {
        &mut self.nodes[id.to_usize_id()]
    }

    /// Appends a value to the node's data series.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an id of this grid.
    pub fn push_value(&mut self, id: U32Id<BTreeGridNode>, value: TreeGridValue) {
        self.node_mut(id).values.push(value);
    }

    /// Root ids, in insertion order.
    pub fn roots(&self) -> &[U32Id<BTreeGridNode>] {
        &self.roots
    }

    fn push_node(&mut self, label: TreeGridLabel) -> U32Id<BTreeGridNode> {
        // The public ids are u32-wide; refuse to mint one that would
        // truncate.
        u32::try_from(self.nodes.len()).expect("TreeGrid arena is full");
        self.nodes.push(TreeGridNode::new(label)).to_u32_id()
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGrid, TreeGridCellFormat, TreeGridLabel, TreeGridValue};

    #[test]
    fn adds_roots_in_insertion_order() {
        let mut grid = TreeGrid::default();
        let first = grid.add_root(TreeGridLabel::bare("0"));
        let second = grid.add_root(TreeGridLabel::bare("1"));

        assert_eq!(grid.roots(), &[first, second]);
        assert_eq!(grid.node(first).label, TreeGridLabel::bare("0"));
        assert_eq!(grid.node(second).label, TreeGridLabel::bare("1"));
    }

    #[test]
    fn attaches_children_in_insertion_order() {
        let mut grid = TreeGrid::default();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let first = grid.add_child(root, TreeGridLabel::quoted("energy-tank-1"));
        let second = grid.add_child(root, TreeGridLabel::quoted("energy-tank-2"));
        let grandchild = grid.add_child(first, TreeGridLabel::bare("transform"));

        assert_eq!(grid.node(root).children(), &[first, second]);
        assert_eq!(grid.node(first).children(), &[grandchild]);
        assert!(grid.node(second).children().is_empty());
    }

    #[test]
    fn a_new_node_is_empty_with_the_auto_format() {
        let mut grid = TreeGrid::default();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let node = grid.node(root);

        assert_eq!(node.annotation, None);
        assert_eq!(node.format, TreeGridCellFormat::Auto);
        assert!(node.values.is_empty());
        assert!(node.children().is_empty());
    }

    #[test]
    fn pushes_values_in_order() {
        let mut grid = TreeGrid::default();
        let root = grid.add_root(TreeGridLabel::bare("0"));
        grid.push_value(root, TreeGridValue::new("255"));
        grid.push_value(root, TreeGridValue::new("128"));

        let texts: Vec<&str> = grid
            .node(root)
            .values
            .iter()
            .map(|value| value.text.as_str())
            .collect();
        assert_eq!(texts, ["255", "128"]);
    }

    #[test]
    fn node_mut_edits_annotation_format_and_values() {
        let mut grid = TreeGrid::default();
        let root = grid.add_root(TreeGridLabel::bare("energy-tank"));
        let node = grid.node_mut(root);
        node.annotation = Some("(Group)".to_owned());
        node.format = TreeGridCellFormat::Text;

        assert_eq!(grid.node(root).annotation.as_deref(), Some("(Group)"));
        assert_eq!(grid.node(root).format, TreeGridCellFormat::Text);
    }

    #[test]
    fn ids_are_distinct_across_the_forest() {
        let mut grid = TreeGrid::default();
        let root = grid.add_root(TreeGridLabel::bare("root"));
        let child = grid.add_child(root, TreeGridLabel::bare("transform"));
        let sibling_root = grid.add_root(TreeGridLabel::bare("unplaced"));

        assert_ne!(root, child);
        assert_ne!(root, sibling_root);
        assert_ne!(child, sibling_root);
        assert_eq!(grid.roots(), &[root, sibling_root]);
    }
}
