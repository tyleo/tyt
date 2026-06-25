use clap::ValueEnum;

/// Output container and printing form for a `to-voxj` document.
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
