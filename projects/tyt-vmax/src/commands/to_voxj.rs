use crate::{
    Dependencies, Result, VoxjEncoding, VoxjFormat, VoxjOptimize, VoxjPositionEncoding,
    VoxjSampleEncoding,
};
use clap::Parser;
use std::path::PathBuf;

/// Converts a `.vmax` package to a Voxel Json document, written to stdout.
///
/// `--format` chooses the output form (compact `.voxj`, `.voxjz` zip, or
/// pretty-printed `.voxj`). The block encodings come from `--position-encoding`
/// and `--sample-encoding`, or from `--optimize` (which picks them
/// automatically and may not be combined with the explicit encoding flags).
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxj")]
pub struct ToVoxj {
    /// The input `.vmax` directory to convert.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Output form: `json` (compact), `zip` (`.voxjz`), or `pretty`.
    #[arg(value_name = "format", long, default_value = "json")]
    format: VoxjFormat,

    /// Position-block encoding. Ignored when `--optimize` is given.
    #[arg(
        value_name = "position-encoding",
        long,
        default_value = "bitmap-base64",
        conflicts_with = "optimize"
    )]
    position_encoding: VoxjPositionEncoding,

    /// Sample-block encoding. Ignored when `--optimize` is given.
    #[arg(
        value_name = "sample-encoding",
        long,
        default_value = "rle-json",
        conflicts_with = "optimize"
    )]
    sample_encoding: VoxjSampleEncoding,

    /// Automatically choose encodings: `size`, `fast`, or `pretty`. Cannot be
    /// combined with `--position-encoding`/`--sample-encoding`.
    #[arg(value_name = "optimize", long)]
    optimize: Option<VoxjOptimize>,
}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let encoding = match self.optimize {
            Some(VoxjOptimize::Size) => VoxjEncoding::Smallest,
            Some(VoxjOptimize::Fast) => VoxjEncoding::Fixed {
                position: VoxjPositionEncoding::BitmapBase64,
                sample: VoxjSampleEncoding::PackedBase64,
            },
            Some(VoxjOptimize::Pretty) => VoxjEncoding::Fixed {
                position: VoxjPositionEncoding::RawJson,
                sample: VoxjSampleEncoding::RawJson,
            },
            None => VoxjEncoding::Fixed {
                position: self.position_encoding,
                sample: self.sample_encoding,
            },
        };
        dependencies.write_voxj(&self.input_vmax, encoding, self.format)
    }
}
