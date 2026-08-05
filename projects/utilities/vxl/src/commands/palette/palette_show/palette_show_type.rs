use clap::ValueEnum;

/// An asserted reading for the custom properties a `palette show` invocation
/// selects. A glTF vocabulary name classifies by name.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowType {
    /// Read a custom `vec-3-float` or `vec-4-float` value pool as a color.
    #[value(name = "color")]
    Color,
}
