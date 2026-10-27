/// What voxelizing does with a material value outside the range the glTF spec
/// gives its property, such as a `metallic` above `1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutOfRangeProperty {
    /// Reject the mesh, naming the property and its value.
    #[default]
    Error,

    /// Clamp the value onto the range and voxelize on.
    Clamp,
}
