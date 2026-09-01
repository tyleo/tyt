/// The palette a [`PropertySelector`](crate::PropertySelector) names: one
/// palette by index, or every palette.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PaletteRef {
    /// Every palette in the document.
    #[default]
    All,

    /// One palette by its index into the document's palettes.
    Index(usize),
}
