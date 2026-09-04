/// A voxel file format this crate converts, as a report names the source a
/// state was read from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoxelFormat {
    /// Voxel Json, the `.voxj` and `.voxjz` documents.
    Voxj,

    /// Voxel Max, the `.vmax` package directory.
    VMax,

    /// MagicaVoxel, the `.vox` file.
    MVox,

    /// Goxel, the `.gox` file.
    Goxl,

    /// Qubicle, the `.qbcl` file.
    Qbcl,
}

impl VoxelFormat {
    /// The short lowercase name, as the codec features spell it.
    pub fn name(self) -> &'static str {
        match self {
            VoxelFormat::Voxj => "voxj",
            VoxelFormat::VMax => "vmax",
            VoxelFormat::MVox => "mvox",
            VoxelFormat::Goxl => "goxl",
            VoxelFormat::Qbcl => "qbcl",
        }
    }
}
