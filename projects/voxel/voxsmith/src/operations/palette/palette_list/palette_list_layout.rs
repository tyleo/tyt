/// How [`palette_list`](crate::operations::palette::palette_list()) lays out
/// the listing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteListLayout {
    /// Indented tree, one palette per branch.
    BoxHierarchy,

    /// A `palettes` line over one aligned record table drawn with box
    /// glyphs, one row per palette.
    BoxTables,

    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A `# palettes` heading over one aligned record table, one row per
    /// palette.
    MdTables,
}
