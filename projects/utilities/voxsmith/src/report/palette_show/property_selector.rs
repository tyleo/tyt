use crate::{PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef};

/// A selector naming one or more value collections for
/// [`render_palette_show`](crate::render_palette_show): a property's values
/// down a palette, with how each value renders. The default names every
/// property of every palette under the `Auto` presentation and reading.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertySelector {
    /// Which palette, one index or every palette.
    pub palette: PaletteRef,

    /// Which property, one key with an optional vector component or every
    /// property.
    pub property: PropertyRef,

    /// What renders for each value in the value collection.
    pub presentation: PaletteShowPresentation,

    /// How each value's numbers spell.
    pub reading: PaletteShowReading,
}
