use clap::ValueEnum;

/// Sample-block encoding for a `to voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjSampleEncoding {
    /// Search for the smallest deflated result.
    #[value(name = "smallest")]
    Smallest,
    /// Raw per-palette sample channels in JSON. The most readable.
    #[value(name = "raw-json", alias = "prettiest")]
    RawJson,
    /// Run-length-encoded per-palette sample channels in JSON.
    #[value(name = "rle-json")]
    RleJson,
    /// Base64 bit-packed samples. The fastest to decode.
    #[value(name = "packed-base64", alias = "fastest")]
    PackedBase64,
}
