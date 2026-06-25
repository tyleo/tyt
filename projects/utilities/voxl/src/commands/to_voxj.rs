use crate::{
    Dependencies, Format, Result, VoxjFormat, VoxjOptimize, VoxjPositionEncoding,
    VoxjSampleEncoding,
};
use clap::Parser;
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
}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-voxj!\n")?;
        Ok(())
    }
}
