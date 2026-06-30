use clap::ValueEnum;

/// Output form for `palette show`.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PaletteShowFormat {
    /// A swatch beside its value for a color, a bare number for a scalar or
    /// extracted channel.
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Swatches alone, with no value text.
    #[value(name = "swatch")]
    Swatch,
    /// Swatches each followed by their value text.
    #[value(name = "swatch-value")]
    SwatchValue,
    /// Raw value text, one per line.
    #[value(name = "value")]
    Value,
}
