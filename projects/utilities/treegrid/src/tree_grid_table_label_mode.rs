/// How `tables` headings spend the ancestor path.
///
/// Tables cannot head their columns with nothing, so unlike
/// [`TreeGridLabelMode`](crate::TreeGridLabelMode) there is no `None`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeGridTableLabelMode {
    /// Nested headings carry full dot-joined paths.
    #[default]
    Concat,

    /// Nested headings carry leaf segments.
    Header,
}
