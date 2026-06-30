use clap::ValueEnum;

/// Output layout for `info`.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum InfoLayout {
    /// Markdown tables: document, palettes, objects.
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
