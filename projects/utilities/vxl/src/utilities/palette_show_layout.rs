use clap::ValueEnum;

/// How `palette show` arranges its value collections and serializes them.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowLayout {
    /// Each collection on one row, separated by a blank line, under headers
    /// padded to the longest so each row's first value aligns. Swatch cells
    /// abut into a strip; other formats put one space between cells.
    #[value(name = "row")]
    Row,

    /// Like `row`, with the header column dropped.
    #[value(name = "row-no-header")]
    RowNoHeader,

    /// Each collection as its own column beneath its header, padded to a common
    /// width.
    #[value(name = "column")]
    Column,

    /// Like `column`, with the header row dropped.
    #[value(name = "column-no-header")]
    ColumnNoHeader,

    /// The collections as an aligned markdown table led by a `#` column of
    /// 0-based material indices, one further column per collection and one row
    /// per material.
    #[value(name = "markdown")]
    Markdown,

    /// The collection records as indented JSON.
    #[value(name = "pretty-json")]
    PrettyJson,

    /// The collection records as single-line JSON.
    #[value(name = "compact-json")]
    CompactJson,
}
