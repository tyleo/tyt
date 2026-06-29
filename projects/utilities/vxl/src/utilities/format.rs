use clap::ValueEnum;
use std::path::Path;

/// A voxel file format that vxl can read or write.
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

impl Format {
    /// Infers the format from `path`'s extension, matched case-insensitively,
    /// or `None` when the extension names no supported format. `.voxj` and
    /// `.voxjz` both map to [`Format::Voxj`].
    pub fn from_path(path: &Path) -> Option<Format> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "voxj" | "voxjz" => Some(Format::Voxj),
            "vmax" => Some(Format::VMax),
            "vox" => Some(Format::MVox),
            "gox" => Some(Format::Goxl),
            "qbcl" => Some(Format::Qbcl),
            _ => None,
        }
    }
}
