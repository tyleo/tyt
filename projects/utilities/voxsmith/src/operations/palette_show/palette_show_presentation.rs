/// What renders for each value in a value collection. A value with no visual,
/// a `bool` for one, renders its text under every presentation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PaletteShowPresentation {
    /// A swatch beside the text for a whole color, else bare text.
    #[default]
    Auto,

    /// Swatches alone, with no value text.
    Swatch,

    /// Each swatch followed by its value text.
    SwatchValue,

    /// Value text alone.
    Value,
}
