use clap::ValueEnum;

/// The pool kind a `--define-attribute` declares for a custom voxel attribute.
/// The mesh command parses its maps before the document loads, so the user
/// states the kind, which sets how a channel may read the value. `color`
/// aliases `srgba` and `scalar` aliases `float`.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum AttributeType {
    /// sRGB color, no alpha. `r`, `g`, `b` components.
    #[value(name = "srgb")]
    Srgb,

    /// sRGB color with straight alpha. `r`, `g`, `b`, `a` components.
    #[value(name = "srgba", alias = "color")]
    Srgba,

    /// Linear-light color, no alpha. `r`, `g`, `b` components.
    #[value(name = "linear-rgb")]
    LinearRgb,

    /// Linear-light color with straight alpha. `r`, `g`, `b`, `a` components.
    #[value(name = "linear-rgba")]
    LinearRgba,

    /// A float scalar, read whole into a 0-1 channel.
    #[value(name = "float", alias = "scalar")]
    Float,

    /// An integer scalar, read whole into a 0-1 channel.
    #[value(name = "int")]
    Int,

    /// A boolean, read as a 1 or 0 mask.
    #[value(name = "bool")]
    Bool,
}

impl AttributeType {
    /// Whether the kind is a color, so a channel reads a component.
    pub fn is_color(self) -> bool {
        matches!(
            self,
            AttributeType::Srgb
                | AttributeType::Srgba
                | AttributeType::LinearRgb
                | AttributeType::LinearRgba
        )
    }

    /// Whether a color kind has an alpha component, so `.a` is valid.
    pub fn has_alpha(self) -> bool {
        matches!(self, AttributeType::Srgba | AttributeType::LinearRgba)
    }
}
