/// How a node's values render to cells in the text layouts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeGridCellFormat {
    /// A color-swatched value shows the swatch beside its text; a
    /// gray-swatched value shows text alone.
    #[default]
    Auto,

    /// A swatched value shows the swatch alone.
    Swatch,

    /// A swatched value shows the swatch beside its text, gray
    /// swatches included.
    SwatchValue,

    /// Text alone.
    Text,
}
