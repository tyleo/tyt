use crate::{
    Dependencies, Format, Result, VoxjEncoding, VoxjFormat, VoxjOptimize, VoxjPositionEncoding,
    VoxjSampleEncoding,
};
use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Converts a voxel file to the Voxel JSON format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxj")]
pub struct ToVoxj {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.voxj` or `.voxjz` document to write.
    #[arg(value_name = "output")]
    output: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Output container and printing form.
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

    /// Pick the block encodings automatically. Conflicts with
    /// `--position-encoding` and `--sample-encoding`.
    #[arg(value_name = "optimize", long)]
    optimize: Option<VoxjOptimize>,

    /// Emit the user-defined `ext` extension block. Pass `--ext false` to omit
    /// it from the output.
    #[arg(
        value_name = "ext",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    ext: bool,
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
        dependencies.to_voxj(
            &self.input,
            self.from,
            &self.output,
            encoding,
            self.format,
            self.ext,
        )
    }
}
