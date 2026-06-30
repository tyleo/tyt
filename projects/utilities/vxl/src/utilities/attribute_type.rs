use clap::ValueEnum;

/// The type of an attribute value: a scalar number or a color.
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum AttributeType {
    /// A single number, clamped to 0-1.
    #[default]
    #[value(name = "scalar")]
    Scalar,
    /// A `#RRGGBBAA` hex color with `r`, `g`, `b`, and `a` components.
    #[value(name = "color")]
    Color,
}
