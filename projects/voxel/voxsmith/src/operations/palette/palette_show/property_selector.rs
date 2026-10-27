use crate::{
    operations::palette::{PaletteShowPresentation, PaletteShowReading, PropertyRef},
    utilities::IdSelector,
};
use voxcore::BVoxPalette;

/// A selector naming one or more value collections for
/// [`palette_show`](crate::operations::palette::palette_show()): a property's
/// values down a palette, with how each value renders. The default names every
/// property of every palette under the `Auto` presentation and reading.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PropertySelector {
    /// Which palettes, by id.
    pub palette: IdSelector<BVoxPalette>,

    /// Which property, one key with an optional vector component or every
    /// property.
    pub property: PropertyRef,

    /// What renders for each value in the value collection.
    pub presentation: PaletteShowPresentation,

    /// How each value's numbers spell.
    pub reading: PaletteShowReading,
}
