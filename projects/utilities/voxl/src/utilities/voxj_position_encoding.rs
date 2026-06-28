use clap::ValueEnum;

/// Position-block encoding for a `to-voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjPositionEncoding {
    /// Raw `[x, y, z]` triples in JSON.
    #[value(name = "raw-json")]
    RawJson,
    /// A base64 occupancy bitmap over the object's bounds.
    #[value(name = "bitmap-base64")]
    BitmapBase64,
    /// Base64 varint deltas along a Hilbert curve.
    #[value(name = "hilbert-delta-varint-base64")]
    Hilbert,
}
