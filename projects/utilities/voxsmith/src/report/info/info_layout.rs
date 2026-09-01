/// How [`render_info`](crate::render_info) lays out the report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoLayout {
    /// A file-name title over `document`, `palettes`, and `objects` record
    /// tables.
    Tables,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// Compact, single-line JSON.
    JsonCompact,
}
