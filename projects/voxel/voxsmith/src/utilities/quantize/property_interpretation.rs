/// How a quantized property's values read as clustering points.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyInterpretation {
    /// `baseColor` and `emissiveColor` read as
    /// [`LinearColor`](Self::LinearColor) and any other property as
    /// [`Numeric`](Self::Numeric).
    Auto,

    /// A 3- or 4-float vector holding a linear-light color.
    LinearColor,

    /// Any float, int, or vector value, compared by its raw components.
    Numeric,

    /// A 3- or 4-float vector holding an sRGB-encoded color.
    SrgbColor,
}
