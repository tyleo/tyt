use clap::ValueEnum;

/// A `--define-attribute` value's kind: a scalar number or a color. The mesh
/// command parses its maps before loading the document, so this is the user's
/// declaration of whether a channel expression must name a color component. The
/// finer pool kind (its color space, or `int` versus `float`) and bounds are
/// resolved at bake time and surfaced by `palette show`, not here.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum AttributeType {
    /// A single number, read whole and packed into a 0-1 channel.
    #[value(name = "scalar")]
    Scalar,

    /// A color, read one `r`, `g`, `b`, or `a` component at a time.
    #[value(name = "color")]
    Color,
}
