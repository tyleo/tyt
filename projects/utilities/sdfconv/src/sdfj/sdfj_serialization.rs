/// How an SDF Json document is serialized: which encoder writes the bytes,
/// and the extension the file takes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SdfjSerialization {
    /// Compact JSON, a `.sdfj` file.
    #[default]
    Compact,

    /// Pretty-printed JSON, a `.sdfj` file.
    Pretty,

    /// Compact JSON deflated into a `.sdfjz` zip archive.
    Zip,
}

impl SdfjSerialization {
    /// The serialization a file extension implies, matched
    /// case-insensitively, or `None` for any other extension. `sdfj` implies
    /// compact because pretty shares the extension and is never inferred.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "sdfj" => Some(SdfjSerialization::Compact),
            "sdfjz" => Some(SdfjSerialization::Zip),
            _ => None,
        }
    }

    /// The extension a document in this serialization takes.
    pub fn extension(self) -> &'static str {
        match self {
            SdfjSerialization::Compact | SdfjSerialization::Pretty => "sdfj",
            SdfjSerialization::Zip => "sdfjz",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sdfj::SdfjSerialization;

    #[test]
    fn extensions_imply_compact_or_zip() {
        assert_eq!(
            SdfjSerialization::from_extension("SDFJ"),
            Some(SdfjSerialization::Compact)
        );

        assert_eq!(
            SdfjSerialization::from_extension("sdfjz"),
            Some(SdfjSerialization::Zip)
        );

        assert_eq!(SdfjSerialization::from_extension("json"), None);

        assert_eq!(SdfjSerialization::Pretty.extension(), "sdfj");

        assert_eq!(SdfjSerialization::Zip.extension(), "sdfjz");
    }
}
