use clap::ValueEnum;

/// Material-map atlas layout: how baked maps are arranged and indexed by the
/// mesh's UVs.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Atlas {
    /// One texel per palette entry, placed at the entry's palette index, shared
    /// by every mesh on the palette.
    #[value(name = "palette")]
    Palette,

    /// A per-mesh UV unwrap with its own texel per face.
    #[value(name = "unwrap")]
    Unwrap,
}
