use clap::ValueEnum;

/// How `palette show` arranges its value collections and serializes them.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowLayout {
    /// The value collections as a box-glyph tree of palettes, properties, and
    /// components, each value collection's values inline on its node.
    #[value(name = "hierarchy")]
    Hierarchy,

    /// Each value collection on one row, separated by a blank line, under labels
    /// padded to the longest so each row's first value aligns. Swatch cells
    /// abut into a strip; other formats put one space between cells.
    #[value(name = "rows")]
    Rows,

    /// Each value collection as its own column beneath its label, padded to a
    /// common width.
    #[value(name = "columns")]
    Columns,

    /// The value collections as aligned markdown tables led by a `#` column of
    /// 0-based material indices; `--table-shape` picks per-palette tables
    /// under headings or one flat comparison table.
    #[value(name = "tables")]
    Tables,

    /// The value collection tree as indented JSON records.
    #[value(name = "json-pretty")]
    JsonPretty,

    /// The value collection tree as single-line JSON records.
    #[value(name = "json-compact")]
    JsonCompact,
}
