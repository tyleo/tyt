/// The arrangement a render produces.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeGridLayout {
    /// A box-glyph tree: annotations show and values print inline
    /// after the label.
    Hierarchy,

    /// One row per data node, labels padded to align, a blank line
    /// between rows.
    #[default]
    Rows,

    /// One padded column per data node.
    Columns,

    /// Aligned markdown tables led by an index column.
    Tables,

    /// The JSON record envelope, pretty-printed.
    JsonPretty,

    /// The JSON record envelope, compact.
    JsonCompact,
}
