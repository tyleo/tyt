/// The table shape. The box tables render takes it as is; the markdown
/// tables render pairs it with heading options as `TreeGridTableShape`.
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
