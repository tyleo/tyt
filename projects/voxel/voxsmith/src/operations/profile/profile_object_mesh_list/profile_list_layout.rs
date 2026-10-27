/// How [`profile_list`](crate::operations::profile::profile_list()) lays out
/// the listing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileListLayout {
    /// A box-glyph tree, one branch per origin over its profiles.
    BoxHierarchy,

    /// One box-glyph table, a column per origin over its profiles.
    BoxTables,

    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A `# profiles` heading over one section per origin, each a numbered
    /// markdown list of its profiles.
    MdLists,

    /// One markdown table, a column per origin over its profiles.
    MdTables,

    /// One row per origin, its profiles beside it.
    TextRows,
}
