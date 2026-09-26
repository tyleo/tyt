use crate::operations::voxelize::ResolutionReference;

/// How the voxel size is chosen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridResolution {
    /// The edge length of one voxel, in the mesh's units.
    VoxelSize(f64),

    /// A reference side divided into `count` voxels.
    ReferenceCount {
        /// The side to divide.
        reference: ResolutionReference,

        /// Voxels along the side.
        count: u32,
    },
}
