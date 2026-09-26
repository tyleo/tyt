/// What happens to a placing node's scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelScale {
    /// The scale is applied to the geometry, so every object's cubes are the
    /// voxel size.
    Bake,

    /// The scale stays on the node, so a scaled object's cubes are the voxel
    /// size times its scale.
    Keep,
}
