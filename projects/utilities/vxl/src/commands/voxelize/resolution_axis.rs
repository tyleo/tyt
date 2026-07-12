use clap::ValueEnum;

/// Which axis a `--resolution` count sizes; the other axes preserve aspect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ResolutionAxis {
    /// The mesh's longest extent.
    #[value(name = "long")]
    Long,

    /// The mesh's shortest extent.
    #[value(name = "short")]
    Short,

    /// The x axis.
    #[value(name = "x")]
    X,

    /// The y axis.
    #[value(name = "y")]
    Y,

    /// The z axis.
    #[value(name = "z")]
    Z,
}
