use crate::{
    Dependencies, EditState, Format, Result, VoxjEncoding, VoxjFormat, VoxjOptimize,
    VoxjPositionEncoding, VoxjSampleEncoding,
};
use clap::{ArgAction, Parser};
use std::path::{Path, PathBuf};

/// Converts a voxel file to the Voxel JSON format.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxj")]
pub struct Voxj {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Output container and printing form. Defaults to compact JSON, or the
    /// container inferred from the output extension when `--format` is omitted.
    #[arg(value_name = "format", long)]
    format: Option<VoxjFormat>,

    /// Position-block encoding. When unset it follows `--optimize`, defaulting
    /// to `smallest`. An explicit value pins the block, leaving `--optimize` to
    /// search only the other.
    #[arg(value_name = "position-encoding", long)]
    position_encoding: Option<VoxjPositionEncoding>,

    /// Sample-block encoding. When unset it follows `--optimize`, defaulting to
    /// `smallest`. An explicit value pins the block, leaving `--optimize` to
    /// search only the other.
    #[arg(value_name = "sample-encoding", long)]
    sample_encoding: Option<VoxjSampleEncoding>,

    /// Default encoding strategy for blocks left unset. An explicit
    /// `--position-encoding` or `--sample-encoding` overrides it for that
    /// block.
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

    /// When to record each object's editor build volume in the document's
    /// `edit_state`. `auto` records it only when some object carries margin
    /// around its live voxels.
    #[arg(value_name = "edit-state", long, default_value = "auto")]
    edit_state: EditState,
}

impl Voxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let encoding = VoxjEncoding {
            position: self
                .position_encoding
                .unwrap_or_else(|| default_position(self.optimize)),
            sample: self
                .sample_encoding
                .unwrap_or_else(|| default_sample(self.optimize)),
        };
        let format = resolve_format(self.format, self.output.as_deref());
        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension(format.extension()));
        dependencies.to_voxj(
            &self.input,
            self.from,
            &output,
            encoding,
            format,
            self.ext,
            self.edit_state,
        )
    }
}

/// The output container: the explicit `--format`, else the container inferred
/// from the output path's extension, else the compact JSON default.
fn resolve_format(format: Option<VoxjFormat>, output: Option<&Path>) -> VoxjFormat {
    format
        .or_else(|| output.and_then(VoxjFormat::from_path))
        .unwrap_or(VoxjFormat::Json)
}

/// The position encoding an unset `--position-encoding` falls back to, derived
/// from `--optimize`. With no `--optimize` it searches for the smallest.
fn default_position(optimize: Option<VoxjOptimize>) -> VoxjPositionEncoding {
    match optimize {
        None | Some(VoxjOptimize::Size) => VoxjPositionEncoding::Smallest,
        Some(VoxjOptimize::Fast) => VoxjPositionEncoding::BitmapBase64,
        Some(VoxjOptimize::Pretty) => VoxjPositionEncoding::RawJson,
    }
}

/// The sample encoding an unset `--sample-encoding` falls back to, derived from
/// `--optimize`. With no `--optimize` it searches for the smallest.
fn default_sample(optimize: Option<VoxjOptimize>) -> VoxjSampleEncoding {
    match optimize {
        None | Some(VoxjOptimize::Size) => VoxjSampleEncoding::Smallest,
        Some(VoxjOptimize::Fast) => VoxjSampleEncoding::PackedBase64,
        Some(VoxjOptimize::Pretty) => VoxjSampleEncoding::RawJson,
    }
}
