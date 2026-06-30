use clap::ValueEnum;

/// How a read-only report renders: human-readable Markdown or JSON. Shared by
/// the report commands so `--layout` means the same everywhere.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ReportLayout {
    /// Human-readable Markdown.
    #[default]
    #[value(name = "markdown")]
    Markdown,
    /// Pretty-printed, multi-line JSON.
    #[value(name = "pretty-json")]
    PrettyJson,
    /// Compact, single-line JSON.
    #[value(name = "compact-json")]
    CompactJson,
}
