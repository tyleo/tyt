/// How the `Tables` layout of
/// [`palette_show`](crate::operations::palette_show::palette_show()) shapes its
/// tables.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteShowTableShape {
    /// One table per palette group, under nested headings.
    Nested,

    /// One table over every value collection, the cross-palette comparison
    /// view.
    Flat,

    /// One row per property under each palette's heading, component values
    /// in relative-path columns.
    Records,
}
