/// How [`render_palette_list`](crate::render_palette_list) lays out the
/// listing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteListLayout {
    /// Indented tree, one palette per branch.
    Hierarchy,

    /// A `# palettes` heading over one aligned record table, one row per
    /// palette.
    Tables,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// Compact, single-line JSON.
    JsonCompact,
}
