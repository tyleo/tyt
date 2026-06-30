use clap::ValueEnum;

/// Material-map atlas layout: how the baked maps are arranged and how the mesh's
/// UVs index them. `palette` depends only on the palette, so every mesh on a
/// palette gets a byte-identical set of maps and shares them. `unwrap` trades
/// that sharing for a per-mesh UV unwrap that can hold spatially varying bakes a
/// single texel per material cannot.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Atlas {
    /// One texel per palette entry, placed at the entry's palette index, shared
    /// by every mesh on the palette.
    #[default]
    #[value(name = "palette")]
    Palette,
    /// A per-mesh UV unwrap with its own texel per face.
    #[value(name = "unwrap")]
    Unwrap,
}
