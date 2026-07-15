/// The table shape.
///
/// The loose counterpart of
/// [`TreeGridTableShape`](crate::TreeGridTableShape).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridTableShapeKind {
    /// One table per parent-path group, under nested headings.
    Nested,

    /// One table over every data node.
    Flat,
}
