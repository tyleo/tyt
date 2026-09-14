use crate::operations::mesh::ArrayDomain;

/// What computes into a binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Computation {
    /// Each entry's index in the domain.
    Index(ArrayDomain),

    /// Occlusion from the voxel geometry.
    Occlusion,

    /// Each voxel's grid position.
    VoxelPosition,
}
