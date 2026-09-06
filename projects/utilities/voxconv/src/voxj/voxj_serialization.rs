/// How a Voxel Json document is serialized: which encoder writes the bytes,
/// and the extension the file takes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VoxjSerialization {
    /// Compact JSON, a `.voxj` file.
    #[default]
    Compact,

    /// Pretty-printed JSON, a `.voxj` file.
    Pretty,

    /// Compact JSON deflated into a `.voxjz` zip archive.
    Zip,
}

impl VoxjSerialization {
    /// The serialization a file extension implies, matched
    /// case-insensitively, or `None` for any other extension. `voxj` implies
    /// compact because pretty shares the extension and is never inferred.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "voxj" => Some(VoxjSerialization::Compact),
            "voxjz" => Some(VoxjSerialization::Zip),
            _ => None,
        }
    }

    /// The extension a document in this serialization takes.
    pub fn extension(self) -> &'static str {
        match self {
            VoxjSerialization::Compact | VoxjSerialization::Pretty => "voxj",
            VoxjSerialization::Zip => "voxjz",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::voxj::VoxjSerialization;

    #[test]
    fn extensions_imply_compact_or_zip() {
        assert_eq!(
            VoxjSerialization::from_extension("VOXJ"),
            Some(VoxjSerialization::Compact)
        );

        assert_eq!(
            VoxjSerialization::from_extension("voxjz"),
            Some(VoxjSerialization::Zip)
        );

        assert_eq!(VoxjSerialization::from_extension("json"), None);

        assert_eq!(VoxjSerialization::Pretty.extension(), "voxj");
    }
}
