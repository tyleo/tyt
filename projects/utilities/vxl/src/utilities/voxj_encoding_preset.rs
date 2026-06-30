use clap::ValueEnum;

/// Default block-encoding strategy for a voxj document, the `--encoding-preset`
/// value.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjEncodingPreset {
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
