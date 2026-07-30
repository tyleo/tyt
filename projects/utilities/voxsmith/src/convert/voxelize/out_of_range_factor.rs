/// What voxelizing does with a source factor outside the range the glTF spec
/// gives its attribute, such as a `metallicFactor` above `1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutOfRangeFactor {
    /// Reject the mesh, naming the factor and its value.
    #[default]
    Error,

    /// Clamp the factor to the range and voxelize on.
    Clamp,
}
