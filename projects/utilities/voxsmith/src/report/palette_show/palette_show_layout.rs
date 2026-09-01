/// How [`render_palette_show`](crate::render_palette_show) arranges its value
/// collections and serializes them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaletteShowLayout {
    /// The value collections as a box-glyph tree of palettes, properties, and
    /// components, each value collection's values inline on its node.
    Hierarchy,

    /// Each value collection on one row, separated by a blank line, under
    /// labels padded to the longest so each row's first value aligns. Swatch
    /// cells abut into a strip; other formats put one space between cells.
    #[default]
    Rows,

    /// Each value collection as its own column beneath its label, padded to a
    /// common width.
    Columns,

    /// The value collections as aligned markdown tables led by a `#` column
    /// of 0-based material indices, shaped by
    /// [`PaletteShowTableShape`](crate::PaletteShowTableShape).
    Tables,

    /// The value collection tree as indented JSON records.
    JsonPretty,

    /// The value collection tree as single-line JSON records.
    JsonCompact,
}
