/// How [`validate`](crate::operations::vox_doc::validate()) lays out the
/// report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidateLayout {
    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A file-name heading over one line per check and a closing pass/fail
    /// summary.
    MdTables,
}
