use clap::ValueEnum;

/// How a read-only report renders: human-readable Markdown or JSON.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReportLayout {
    /// Human-readable Markdown.
    #[value(name = "markdown")]
    Markdown,

    /// Pretty-printed, multi-line JSON.
    #[value(name = "pretty-json")]
    PrettyJson,

    /// Compact, single-line JSON.
    #[value(name = "compact-json")]
    CompactJson,
}
