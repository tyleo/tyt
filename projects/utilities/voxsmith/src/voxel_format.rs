/// A voxel file format this crate converts, as a report names the source a
/// state was read from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoxelFormat {
    /// Voxel Json, the `.voxj` and `.voxjz` documents.
    VoxelJson,

    /// Voxel Max, the `.vmax` package directory.
    VoxelMax,

    /// MagicaVoxel, the `.vox` file.
    MagicaVoxel,

    /// Goxel, the `.gox` file.
    Goxel,

    /// Qubicle, the `.qbcl` file.
    QubicleQbcl,
}

impl VoxelFormat {
    /// The short lowercase name, as the codec features spell it.
    pub fn name(self) -> &'static str {
        match self {
            VoxelFormat::VoxelJson => "voxj",
            VoxelFormat::VoxelMax => "vmax",
            VoxelFormat::MagicaVoxel => "mvox",
            VoxelFormat::Goxel => "goxl",
            VoxelFormat::QubicleQbcl => "qbcl",
        }
    }
}
