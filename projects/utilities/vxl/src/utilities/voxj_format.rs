use clap::ValueEnum;
use std::path::Path;

/// Output container and printing form for a `to voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjFormat {
    /// Compact `.voxj` JSON.
    #[value(name = "json")]
    Json,
    /// Compressed `.voxjz` zip archive.
    #[value(name = "zip")]
    Zip,
    /// Pretty-printed `.voxj` JSON.
    #[value(name = "pretty")]
    PrettyJson,
}

impl VoxjFormat {
    /// Infers the output container from `path`'s extension, matched
    /// case-insensitively, or `None` when the extension names no voxj
    /// container. `.voxj` selects compact JSON; `.voxjz` selects the zip
    /// archive. The pretty-printed container shares the `.voxj` extension and
    /// is never inferred.
    pub fn from_path(path: &Path) -> Option<VoxjFormat> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "voxj" => Some(VoxjFormat::Json),
            "voxjz" => Some(VoxjFormat::Zip),
            _ => None,
        }
    }

    /// The extension a defaulted output path takes for this container. The two
    /// JSON containers write `.voxj`; the zip container writes `.voxjz`.
    pub fn extension(self) -> &'static str {
        match self {
            VoxjFormat::Json | VoxjFormat::PrettyJson => "voxj",
            VoxjFormat::Zip => "voxjz",
        }
    }
}
