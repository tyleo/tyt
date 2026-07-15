use crate::{
    TreeGridColumnsOptions, TreeGridHierarchyOptions, TreeGridRowsOptions, TreeGridTableShape,
};

/// A validated render request: the layout, carrying only the options
/// it consumes.
///
/// Built directly, or from flag-shaped input through
/// [`TreeGridOptions::resolve`](crate::TreeGridOptions::resolve).
/// Invalid option combinations are unrepresentable, so a render never
/// error-checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridLayout {
    /// The box-glyph tree.
    Hierarchy(TreeGridHierarchyOptions),

    /// One row per data node, labels padded to align, a blank line
    /// between rows.
    Rows(TreeGridRowsOptions),

    /// One padded column per data node.
    Columns(TreeGridColumnsOptions),

    /// Aligned markdown tables led by an index column.
    Tables(TreeGridTableShape),

    /// The JSON record envelope, pretty-printed.
    #[cfg(feature = "json")]
    JsonPretty,

    /// The JSON record envelope, compact.
    #[cfg(feature = "json")]
    JsonCompact,
}

impl Default for TreeGridLayout {
    fn default() -> Self {
        Self::Rows(TreeGridRowsOptions::default())
    }
}
