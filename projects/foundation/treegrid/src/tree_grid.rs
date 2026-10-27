use crate::{BTreeGridNode, TreeGridCells, TreeGridLabel, TreeGridNode, TreeGridValueCells};
use branded_id::{IdVec, U32Id};
use std::fmt::{Debug, Formatter, Result as FmtResult};

/// An ordered forest of labeled, data-bearing nodes in an append-only
/// arena, rendered through a cell policy.
///
/// Populate with [`retain_root`](Self::retain_root) and
/// [`retain_child`](Self::retain_child), then attach data with
/// [`push_value`](Self::push_value) and edit nodes through
/// [`node_mut`](Self::node_mut). Nodes attach to their parent at
/// creation and are never released or re-parented, so the forest cannot
/// cycle. Ids are dense indices into this grid and are meaningful only
/// within it.
///
/// The policy `C` decides how the stored values render to cells:
/// [`new`](Self::new) builds a grid over the default
/// [`TreeGridValueCells`] and pre-rendered
/// [`TreeGridValue`](crate::TreeGridValue)s, and
/// [`with_cells`](Self::with_cells) builds one over a custom policy
/// and its own value type.
pub struct TreeGrid<C: TreeGridCells = TreeGridValueCells> {
    /// The cell policy the renders consult.
    cells: C,

    /// Dense node storage; a node's id is its index.
    nodes: IdVec<BTreeGridNode, TreeGridNode<C::Value>>,

    /// Root ids, in insertion order.
    roots: Vec<U32Id<BTreeGridNode>>,
}

impl TreeGrid {
    /// Creates an empty grid over the default cell policy.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C: TreeGridCells> TreeGrid<C> {
    /// Creates an empty grid over `cells`.
    pub fn with_cells(cells: C) -> Self {
        Self {
            cells,
            nodes: IdVec::new(),
            roots: Vec::new(),
        }
    }

    /// The cell policy the renders consult.
    pub fn cells(&self) -> &C {
        &self.cells
    }

    /// Retains a root node and returns its id.
    pub fn retain_root(&mut self, label: TreeGridLabel) -> U32Id<BTreeGridNode> {
        let id = self.push_node(label);
        self.roots.push(id);
        id
    }

    /// Retains a child under `parent` and returns its id.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is not an id of this grid.
    pub fn retain_child(
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
    pub fn node(&self, id: U32Id<BTreeGridNode>) -> &TreeGridNode<C::Value> {
        &self.nodes[id.to_usize_id()]
    }

    /// The node behind `id`, mutably. Children are not reachable
    /// through it; they attach only at creation.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an id of this grid.
    pub fn node_mut(&mut self, id: U32Id<BTreeGridNode>) -> &mut TreeGridNode<C::Value> {
        &mut self.nodes[id.to_usize_id()]
    }

    /// Appends a value to the node's data series.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not an id of this grid.
    pub fn push_value(&mut self, id: U32Id<BTreeGridNode>, value: C::Value) {
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

// Manual impls: a derive would bound only `C`, not the stored
// `C::Value`.

impl<C: TreeGridCells + Clone> Clone for TreeGrid<C>
where
    C::Value: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            nodes: self.nodes.clone(),
            roots: self.roots.clone(),
        }
    }
}

impl<C: TreeGridCells + Debug> Debug for TreeGrid<C>
where
    C::Value: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("TreeGrid")
            .field("cells", &self.cells)
            .field("nodes", &self.nodes)
            .field("roots", &self.roots)
            .finish()
    }
}

impl<C: TreeGridCells + Default> Default for TreeGrid<C> {
    fn default() -> Self {
        Self::with_cells(C::default())
    }
}

impl<C: TreeGridCells + PartialEq> PartialEq for TreeGrid<C>
where
    C::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cells == other.cells && self.nodes == other.nodes && self.roots == other.roots
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGrid, TreeGridCellFormat, TreeGridCells, TreeGridLabel, TreeGridValue};
    use std::borrow::Cow;

    #[test]
    fn retains_roots_in_insertion_order() {
        let mut grid = TreeGrid::new();
        let first = grid.retain_root(TreeGridLabel::bare("0"));
        let second = grid.retain_root(TreeGridLabel::bare("1"));

        assert_eq!(grid.roots(), &[first, second]);
        assert_eq!(grid.node(first).label, TreeGridLabel::bare("0"));
        assert_eq!(grid.node(second).label, TreeGridLabel::bare("1"));
    }

    #[test]
    fn attaches_children_in_insertion_order() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        let first = grid.retain_child(root, TreeGridLabel::quoted("energy-tank-1"));
        let second = grid.retain_child(root, TreeGridLabel::quoted("energy-tank-2"));
        let grandchild = grid.retain_child(first, TreeGridLabel::bare("transform"));

        assert_eq!(grid.node(root).children(), &[first, second]);
        assert_eq!(grid.node(first).children(), &[grandchild]);
        assert!(grid.node(second).children().is_empty());
    }

    #[test]
    fn a_new_node_is_empty_with_no_format() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        let node = grid.node(root);

        assert_eq!(node.annotation, None);
        assert_eq!(node.format, None);
        assert!(node.values.is_empty());
        assert!(node.children().is_empty());
    }

    #[test]
    fn pushes_values_in_order() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("0"));
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
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("energy-tank"));
        let node = grid.node_mut(root);
        node.annotation = Some("(Group)".to_owned());
        node.format = Some(TreeGridCellFormat::Text);

        assert_eq!(grid.node(root).annotation.as_deref(), Some("(Group)"));
        assert_eq!(grid.node(root).format, Some(TreeGridCellFormat::Text));
    }

    #[test]
    fn ids_are_distinct_across_the_forest() {
        let mut grid = TreeGrid::new();
        let root = grid.retain_root(TreeGridLabel::bare("root"));
        let child = grid.retain_child(root, TreeGridLabel::bare("transform"));
        let sibling_root = grid.retain_root(TreeGridLabel::bare("unplaced"));

        assert_ne!(root, child);
        assert_ne!(root, sibling_root);
        assert_ne!(child, sibling_root);
        assert_eq!(grid.roots(), &[root, sibling_root]);
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    struct StringCells;

    impl TreeGridCells for StringCells {
        type Value = String;

        fn text<'a>(&self, value: &'a String) -> Cow<'a, str> {
            Cow::Borrowed(value)
        }
    }

    #[test]
    fn a_custom_policy_stores_its_own_value_type() {
        let mut grid = TreeGrid::with_cells(StringCells);
        let root = grid.retain_root(TreeGridLabel::bare("position"));
        grid.push_value(root, "[12.5, 0.5, 10.0]".to_owned());

        let value = &grid.node(root).values[0];
        assert_eq!(grid.cells().text(value), "[12.5, 0.5, 10.0]");
        assert_eq!(grid.cells().visual(value), None);
        assert_eq!(grid.cells().format(value), TreeGridCellFormat::Text);
    }
}
