/// How `voxelize` sizes the voxel grid: exactly one of a voxel count along the
/// longest axis, or a real-world voxel size. The command's clap `ArgGroup` makes
/// `--voxel-grid-length` and `--meters-per-voxel` mutually exclusive and
/// required, and resolves to one of these before handing it on.
#[derive(Clone, Copy, Debug)]
pub enum GridResolution {
    /// Voxels along the longest axis; the other axes are sized to preserve
    /// aspect, leaving the placing node's scale at `1`.
    VoxelGridLength(u32),

    /// Meters per voxel, sizing each axis to a fixed real-world voxel size and
    /// recorded as the placing node's scale.
    MetersPerVoxel(f64),
}
