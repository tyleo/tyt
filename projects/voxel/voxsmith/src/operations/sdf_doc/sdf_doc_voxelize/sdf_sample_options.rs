use crate::utilities::{GridResolution, VoxelFrame};

/// The options [`sample`](crate::operations::sdf_doc::sample()) samples under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfSampleOptions {
    /// How the voxel size is chosen.
    pub resolution: GridResolution,

    /// The frame each part's grid is built in. Under
    /// [`VoxelFrame::World`] each place of a part samples on its own, and
    /// under [`VoxelFrame::Local`] each part samples once for all its places.
    pub frame: VoxelFrame,

    /// Whether each place samples at its pose on the lattice of the nearest
    /// root part or joint above it, which flattening reads. Needs
    /// [`VoxelFrame::World`].
    pub posed: bool,

    /// Whether each step records the cells it wrote and the steps it wrote
    /// over, which the [`report`](crate::operations::sdf_doc::report()) reads.
    pub record_writes: bool,
}
