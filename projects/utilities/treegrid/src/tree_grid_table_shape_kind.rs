/// The table shape.
///
/// The loose counterpart of the tables-feature `TreeGridTableShape`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridTableShapeKind {
    /// One table per parent-path group, under nested headings.
    Nested,

    /// One table over every data node.
    Flat,

    /// One table per root: one row per child, one column per
    /// descendant data path named relative to the row.
    Records,
}
