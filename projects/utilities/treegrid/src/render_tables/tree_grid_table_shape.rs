use crate::TreeGridNestedTableOptions;

/// The `tables` layout's shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridTableShape {
    /// One table per parent-path group, under nested headings.
    Nested(TreeGridNestedTableOptions),

    /// One table over every data node, headed by full concat paths:
    /// the comparison view.
    Flat,
}

impl Default for TreeGridTableShape {
    fn default() -> Self {
        Self::Nested(TreeGridNestedTableOptions::default())
    }
}
