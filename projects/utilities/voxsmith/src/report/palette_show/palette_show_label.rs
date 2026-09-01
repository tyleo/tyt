/// How the text layouts of
/// [`render_palette_show`](crate::render_palette_show) label each value
/// collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteShowLabel {
    /// No labels.
    None,

    /// Full dot-joined paths, like `0."baseColor".a`.
    Concat,

    /// Nested markdown headings over value collections labeled by their leaf
    /// segment alone.
    Header,
}
