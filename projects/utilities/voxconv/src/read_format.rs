/// A voxel document format a document is read as, one variant per enabled
/// format feature.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReadFormat {
    /// Goxel, the `.gox` file.
    #[cfg(feature = "goxl")]
    Goxl,

    /// MagicaVoxel, the `.vox` file.
    #[cfg(feature = "mvox")]
    MVox,

    /// Qubicle Binary, the `.qb` file.
    #[cfg(feature = "qbcl")]
    Qb,

    /// Qubicle Binary Tree, the `.qbt` file.
    #[cfg(feature = "qbcl")]
    Qbt,

    /// Qubicle Construction Library, the `.qbcl` file.
    #[cfg(feature = "qbcl")]
    Qbcl,

    /// Voxel Max, the `.vmax` package directory.
    #[cfg(feature = "vmax")]
    VMax,

    /// Voxel Json, the `.voxj` and `.voxjz` documents.
    #[cfg(feature = "voxj")]
    Voxj,
}

impl ReadFormat {
    /// The format a file extension implies, matched case-insensitively, or
    /// `None` when the extension implies no enabled format. `voxj` and
    /// `voxjz` both imply Voxel Json.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            #[cfg(feature = "goxl")]
            "gox" => Some(ReadFormat::Goxl),
            #[cfg(feature = "mvox")]
            "vox" => Some(ReadFormat::MVox),
            #[cfg(feature = "qbcl")]
            "qb" => Some(ReadFormat::Qb),
            #[cfg(feature = "qbcl")]
            "qbt" => Some(ReadFormat::Qbt),
            #[cfg(feature = "qbcl")]
            "qbcl" => Some(ReadFormat::Qbcl),
            #[cfg(feature = "vmax")]
            "vmax" => Some(ReadFormat::VMax),
            #[cfg(feature = "voxj")]
            "voxj" | "voxjz" => Some(ReadFormat::Voxj),
            _ => None,
        }
    }

    /// The short lowercase name, as the format features spell it, with the
    /// three Qubicle formats told apart by extension.
    pub fn name(self) -> &'static str {
        match self {
            #[cfg(feature = "goxl")]
            ReadFormat::Goxl => "goxl",
            #[cfg(feature = "mvox")]
            ReadFormat::MVox => "mvox",
            #[cfg(feature = "qbcl")]
            ReadFormat::Qb => "qb",
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbt => "qbt",
            #[cfg(feature = "qbcl")]
            ReadFormat::Qbcl => "qbcl",
            #[cfg(feature = "vmax")]
            ReadFormat::VMax => "vmax",
            #[cfg(feature = "voxj")]
            ReadFormat::Voxj => "voxj",
        }
    }

    /// Whether a document in this format is a package directory.
    pub fn is_package(self) -> bool {
        #[cfg(feature = "vmax")]
        let package = self == ReadFormat::VMax;

        #[cfg(not(feature = "vmax"))]
        let package = false;

        package
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadFormat;

    #[test]
    fn extensions_map_case_insensitively() {
        assert_eq!(ReadFormat::from_extension("vox"), Some(ReadFormat::MVox));

        assert_eq!(ReadFormat::from_extension("VOXJZ"), Some(ReadFormat::Voxj));

        assert_eq!(ReadFormat::from_extension("Qbt"), Some(ReadFormat::Qbt));

        assert_eq!(ReadFormat::from_extension("obj"), None);
    }

    #[test]
    fn only_the_vmax_package_is_a_directory() {
        assert!(ReadFormat::VMax.is_package());

        assert!(!ReadFormat::Voxj.is_package());

        assert_eq!(ReadFormat::Qbcl.name(), "qbcl");
    }
}
