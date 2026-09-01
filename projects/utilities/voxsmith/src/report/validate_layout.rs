/// How [`render_validation`](crate::render_validation) lays out the report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidateLayout {
    /// A file-name heading over one line per check and a closing pass/fail
    /// summary.
    Tables,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// Compact, single-line JSON.
    JsonCompact,
}
