use clap::ValueEnum;

/// A voxel file format that voxl can read or write.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Format {
    /// Voxel JSON, the `.voxj` and `.voxjz` documents.
    #[value(name = "voxj")]
    Voxj,
    /// Voxel Max, the `.vmax` package directory.
    #[value(name = "vmax")]
    VMax,
    /// MagicaVoxel, the `.vox` file.
    #[value(name = "mvox")]
    MVox,
    /// Goxel, the `.gox` file.
    #[value(name = "goxl")]
    Goxl,
    /// Qubicle, the `.qbcl` file.
    #[value(name = "qbcl")]
    Qbcl,
}
