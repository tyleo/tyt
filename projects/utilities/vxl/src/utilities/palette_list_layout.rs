use clap::ValueEnum;

/// How `palette list` renders: an indented tree like `hierarchy show`, an
/// aligned Markdown table, or JSON. Its own enum rather than the shared
/// `ReportLayout` because only `palette list` offers the `hierarchy` tree, which
/// is also its default.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PaletteListLayout {
    /// Indented tree, one palette per branch, like `hierarchy show`.
    #[default]
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
