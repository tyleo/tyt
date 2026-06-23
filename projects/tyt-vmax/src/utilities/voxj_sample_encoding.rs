use clap::ValueEnum;

/// Sample-block encoding for a `to-voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjSampleEncoding {
    /// Raw per-voxel sample rows in JSON.
    #[value(name = "raw-json")]
    RawJson,
    /// Run-length-encoded sample rows in JSON.
    #[value(name = "rle-json")]
    RleJson,
    /// Base64 bit-packed samples.
    #[value(name = "packed-base64")]
    PackedBase64,
}
