/// How much of a body a voxelizer keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    /// The body's whole volume. A mesh has to be watertight to enclose one.
    Solid,

    /// Only a hollow shell of the voxels on the body's surface.
    Surface,
}
