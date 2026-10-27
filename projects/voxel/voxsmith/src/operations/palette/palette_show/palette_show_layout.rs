/// How [`palette_show`](crate::operations::palette::palette_show()) arranges
/// its value collections and serializes them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaletteShowLayout {
    /// The value collections as a box-glyph tree of palettes, properties, and
    /// components, each value collection's values inline on its node.
    BoxHierarchy,

    /// The value collections as box-glyph tables led by a `#` column of 0-based
    /// material indices, shaped by
    /// [`PaletteShowTableShape`](crate::operations::palette::PaletteShowTableShape),
    /// each section headed by a bare full-path line.
    BoxTables,

    /// The value collection tree as single-line JSON records.
    JsonCompact,

    /// The value collection tree as indented JSON records.
    JsonPretty,

    /// The value collections as aligned markdown tables led by a `#` column of
    /// 0-based material indices, shaped by
    /// [`PaletteShowTableShape`](crate::operations::palette::PaletteShowTableShape).
    MdTables,

    /// Each value collection as its own column beneath its label, padded to a
    /// common width.
    TextColumns,

    /// Each value collection on one row, separated by a blank line, under
    /// labels padded to the longest so each row's first value aligns. Swatch
    /// cells abut into a strip; other formats put one space between cells.
    #[default]
    TextRows,
}
