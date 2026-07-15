/// Which arrangement to render.
///
/// The loose counterpart of
/// [`TreeGridLayout`](crate::TreeGridLayout);
/// [`TreeGridOptions::resolve`](crate::TreeGridOptions::resolve) maps
/// it into the structural form.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeGridLayoutKind {
    /// The box-glyph tree.
    Hierarchy,

    /// One row per data node.
    #[default]
    Rows,

    /// One padded column per data node.
    Columns,

    /// Aligned markdown tables.
    Tables,

    /// The JSON record envelope, pretty-printed.
    #[cfg(feature = "json")]
    JsonPretty,

    /// The JSON record envelope, compact.
    #[cfg(feature = "json")]
    JsonCompact,
}
