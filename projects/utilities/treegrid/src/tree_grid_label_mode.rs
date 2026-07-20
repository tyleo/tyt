use crate::TreeGridHeaderOptions;

/// How the `rows` and `columns` layouts spend the ancestor path.
///
/// Lives on those layouts' option payloads; tables carry the
/// two-variant tables-feature `TreeGridTableLabelMode` instead, and
/// the `hierarchy` and JSON layouts carry labels structurally with no
/// mode at all.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeGridLabelMode {
    /// No labels anywhere.
    None,

    /// Each data node is labeled by its full dot-joined path, each
    /// quoted segment quoted.
    #[default]
    Concat,

    /// The ancestor chain becomes nested markdown headings; group
    /// content is labeled by leaf segment alone.
    Header(TreeGridHeaderOptions),
}
