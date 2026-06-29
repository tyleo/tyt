use clap::ValueEnum;

/// Position-block encoding for a `to voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjPositionEncoding {
    /// Search for the smallest deflated result.
    #[value(name = "smallest")]
    Smallest,
    /// Raw `[x, y, z]` triples in JSON. The most readable.
    #[value(name = "raw-json", alias = "prettiest")]
    RawJson,
    /// A base64 occupancy bitmap over the object's bounds. The fastest to decode.
    #[value(name = "bitmap-base64", alias = "fastest")]
    BitmapBase64,
    /// Base64 varint deltas along a Hilbert curve.
    #[value(name = "hilbert-delta-varint-base64")]
    Hilbert,
}
