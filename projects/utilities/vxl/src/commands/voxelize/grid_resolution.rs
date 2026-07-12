use crate::commands::ResolutionAxis;

/// How `voxelize` sizes the voxel grid.
#[derive(Clone, Copy, Debug)]
pub enum GridResolution {
    /// A voxel count along a chosen axis; the other axes are sized to preserve
    /// aspect, leaving the placing node's scale at `1`.
    AxisVoxelCount {
        /// Which axis the `count` sizes.
        axis: ResolutionAxis,
        /// Voxels along `axis`.
        count: u32,
    },

    /// Meters per voxel, sizing each axis to a fixed real-world voxel size and
    /// recorded as the placing node's scale.
    MetersPerVoxel(f64),
}
