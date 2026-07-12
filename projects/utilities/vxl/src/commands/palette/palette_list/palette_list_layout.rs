use clap::ValueEnum;

/// How `palette list` renders: an indented tree like `hierarchy show`, an
/// aligned Markdown table, or JSON. Its own enum rather than the shared
/// `ReportLayout` because only `palette list` offers the `hierarchy` tree.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum PaletteListLayout {
    /// Indented tree, one palette per branch, like `hierarchy show`.
    #[value(name = "hierarchy")]
    Hierarchy,

    /// Aligned Markdown table.
    #[value(name = "markdown")]
    Markdown,

    /// Pretty-printed, multi-line JSON.
    #[value(name = "pretty-json")]
    PrettyJson,

    /// Compact, single-line JSON.
    #[value(name = "compact-json")]
    CompactJson,
}
