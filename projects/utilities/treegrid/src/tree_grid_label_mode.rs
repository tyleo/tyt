/// How the `rows`, `columns`, and `tables` layouts spend the ancestor
/// path.
///
/// The `hierarchy` and JSON layouts carry labels structurally and
/// take no mode; setting one there is
/// [`LabelModeWithoutLabels`](crate::TreeGridError::LabelModeWithoutLabels).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridLabelMode {
    /// No labels anywhere. Invalid under `tables`, which cannot head
    /// its columns with nothing
    /// ([`LabelNoneWithTables`](crate::TreeGridError::LabelNoneWithTables)).
    None,

    /// Each data node is labeled by its full dot-joined path, each
    /// quoted segment quoted. The default when no mode is set.
    Concat,

    /// The ancestor chain becomes nested markdown headings; group
    /// content is labeled by leaf segment alone.
    Header,
}
