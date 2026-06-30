use crate::{Dependencies, EditState, Format, Result, VoxjEncodingOptions};
use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Converts a voxel file to the Voxel JSON format.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxj")]
pub struct ToVoxj {
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

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,

    /// Emit the user-defined `ext` extension block. `--ext false` omits it.
    #[arg(
        value_name = "ext",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    ext: bool,

    /// When to record each object's editor build volume. `auto` records it only
    /// when an object has margin around its live voxels.
    #[arg(value_name = "edit-state", long, default_value = "auto")]
    edit_state: EditState,
}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let encoding = self.encoding_options.encoding();
        let format = self.encoding_options.resolve_format(self.output.as_deref());
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
