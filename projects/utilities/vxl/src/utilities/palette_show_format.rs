use clap::ValueEnum;

/// Output form for `palette show`.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PaletteShowFormat {
    /// Swatches for `rgba`, numeric values otherwise.
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Colored swatches.
    #[value(name = "swatch")]
    Swatch,
    /// Raw values, one per line.
    #[value(name = "string")]
    String,
}
