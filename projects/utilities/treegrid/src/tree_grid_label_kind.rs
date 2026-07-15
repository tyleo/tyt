/// How the text layouts spend the ancestor path.
///
/// The loose counterpart of
/// [`TreeGridLabelMode`](crate::TreeGridLabelMode) and
/// [`TreeGridTableLabelMode`](crate::TreeGridTableLabelMode).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridLabelKind {
    /// No labels anywhere.
    None,

    /// Full dot-joined paths.
    Concat,

    /// Nested markdown headings over leaf-labeled groups.
    Header,
}
