use clap::ValueEnum;

/// What `voxelize` does with a source material value outside the range the
/// glTF spec gives its property, such as a `metallic` above `1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutOfRangeProperty {
    /// Reject the mesh, naming the property and its value.
    #[default]
    #[value(name = "error")]
    Error,

    /// Clamp the value onto the range and voxelize on.
    #[value(name = "clamp")]
    Clamp,
}
