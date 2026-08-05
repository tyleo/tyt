use clap::ValueEnum;

/// What renders for each value in a `palette show` value collection. A value
/// with no visual, a `bool` for one, renders its text under every
/// presentation.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowPresentation {
    /// A swatch beside the text for a whole color, else bare text.
    #[value(name = "auto")]
    Auto,

    /// Swatches alone, with no value text.
    #[value(name = "swatch")]
    Swatch,

    /// Each swatch followed by its value text.
    #[value(name = "swatch-value")]
    SwatchValue,

    /// Value text alone, one per line.
    #[value(name = "value")]
    Value,
}
