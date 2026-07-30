use clap::ValueEnum;

/// What `voxelize` does with a source material factor outside the range the
/// glTF spec gives its attribute, such as a `metallicFactor` above `1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutOfRangeFactor {
    /// Reject the mesh, naming the factor and its value.
    #[default]
    #[value(name = "error")]
    Error,

    /// Clamp the factor to the range and voxelize on.
    #[value(name = "clamp")]
    Clamp,
}
