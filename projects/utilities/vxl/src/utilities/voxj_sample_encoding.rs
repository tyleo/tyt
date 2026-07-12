use clap::ValueEnum;

/// Sample-block encoding for a voxj document. Every encoding lays out one
/// channel per layer, each a material index into that layer's palette for every
/// voxel.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjSampleEncoding {
    /// Search for the smallest deflated result.
    #[value(name = "smallest")]
    Smallest,
    /// One raw JSON channel of material indices per layer. The most readable.
    #[value(name = "raw-json", alias = "prettiest")]
    RawJson,
    /// One run-length-encoded JSON channel per layer.
    #[value(name = "rle-json")]
    RleJson,
    /// One bit-packed, base64-encoded channel per layer. The fastest to decode.
    #[value(name = "packed-base64", alias = "fastest")]
    PackedBase64,
}
