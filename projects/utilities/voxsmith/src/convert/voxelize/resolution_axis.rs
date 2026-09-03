/// Which axis a [`GridResolution::AxisVoxelCount`](crate::GridResolution)
/// count sizes; the other axes preserve aspect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionAxis {
    /// The mesh's longest extent.
    Long,

    /// The mesh's shortest extent.
    Short,

    /// The x axis.
    X,

    /// The y axis.
    Y,

    /// The z axis.
    Z,
}
