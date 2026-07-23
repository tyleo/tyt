use clap::ValueEnum;

/// How `validate` renders the report.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ValidateLayout {
    /// A file-name heading over one line per check and a closing pass/fail
    /// summary.
    #[value(name = "tables")]
    Tables,

    /// Pretty-printed, multi-line JSON.
    #[value(name = "json-pretty")]
    JsonPretty,

    /// Compact, single-line JSON.
    #[value(name = "json-compact")]
    JsonCompact,
}
