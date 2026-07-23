use clap::ValueEnum;

/// How `info` renders the report.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum InfoLayout {
    /// A file-name title over `Document`, `Palettes`, and `Objects` record
    /// tables.
    #[value(name = "tables")]
    Tables,

    /// Pretty-printed, multi-line JSON.
    #[value(name = "json-pretty")]
    JsonPretty,

    /// Compact, single-line JSON.
    #[value(name = "json-compact")]
    JsonCompact,
}
