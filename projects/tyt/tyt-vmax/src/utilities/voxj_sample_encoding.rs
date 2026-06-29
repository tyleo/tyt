use clap::ValueEnum;

/// Sample-block encoding for a `to-voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjSampleEncoding {
    /// Raw per-palette sample channels in JSON.
    #[value(name = "raw-json")]
    RawJson,
    /// Run-length-encoded per-palette sample channels in JSON.
    #[value(name = "rle-json")]
    RleJson,
    /// Base64 bit-packed samples.
    #[value(name = "packed-base64")]
    PackedBase64,
}
