use clap::ValueEnum;

/// How `palette list` renders: an aligned Markdown table, an indented tree like
/// `hierarchy show`, or JSON. Its own enum rather than the shared `ReportLayout`
/// because only `palette list` offers the `hierarchy` tree.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PaletteListLayout {
    /// Aligned Markdown table.
    #[default]
    #[value(name = "markdown")]
    Markdown,
    /// Indented tree, one palette per branch, like `hierarchy show`.
    #[value(name = "hierarchy")]
    Hierarchy,
    /// Pretty-printed, multi-line JSON.
    #[value(name = "pretty-json")]
    PrettyJson,
    /// Compact, single-line JSON.
    #[value(name = "compact-json")]
    CompactJson,
}
