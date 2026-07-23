use crate::{TreeGridNestedTableOptions, TreeGridRecordsTableOptions};

/// The `tables` layout's shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridTableShape {
    /// One table per parent-path group, under nested headings.
    Nested(TreeGridNestedTableOptions),

    /// One table over every data node, headed by full concat paths:
    /// the comparison view.
    Flat,

    /// One table per root, transposed from the series shapes: one
    /// row per child rather than per value index, one column per
    /// descendant data path named relative to the row. The entity
    /// listing view.
    Records(TreeGridRecordsTableOptions),
}

impl Default for TreeGridTableShape {
    fn default() -> Self {
        Self::Nested(TreeGridNestedTableOptions::default())
    }
}
