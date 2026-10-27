/// The frame each placed object's voxel grid is built in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelFrame {
    /// World space, the placing node's rotation and translation applied to the
    /// geometry, so every object's voxels align on one lattice and an object
    /// two nodes place voxelizes twice.
    World,

    /// The object's local space, the placing node keeping its rotation and
    /// translation, so an object two nodes place voxelizes once and both
    /// nodes share it.
    Local,
}
