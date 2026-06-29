use clap::ValueEnum;

/// Encoding strategy that fills the default for any block a `to voxj` document
/// leaves unset. An explicit `--position-encoding` or `--sample-encoding`
/// overrides it for that block.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjOptimize {
    /// Search every non-raw encoding pairing and keep the smallest.
    #[value(name = "size")]
    Size,
    /// Fast to decode: bitmap positions and packed samples.
    #[value(name = "fast")]
    Fast,
    /// Most readable: raw positions and raw samples.
    #[value(name = "pretty")]
    Pretty,
}
