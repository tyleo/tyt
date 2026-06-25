use clap::ValueEnum;

/// Automatic encoding strategy that picks the block encodings for a `to-voxj`
/// document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjOptimize {
    /// Try every non-raw encoding pairing and keep the smallest.
    #[value(name = "size")]
    Size,
    /// Fast to decode: bitmap positions and packed samples.
    #[value(name = "fast")]
    Fast,
    /// Most readable: raw positions and raw samples.
    #[value(name = "pretty")]
    Pretty,
}
