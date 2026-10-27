use crate::{VoxjEncodingOptions, cli_value_parser};
use clap::{ArgAction, Args};
use std::path::{Path, PathBuf};
use voxconv::{
    WriteFormat,
    voxj::{EditStateMode, VoxjWriteFormat},
};

/// The voxj document a command writes and how to write it.
#[derive(Clone, Debug, Args)]
pub struct VoxjOutput {
    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` for a `.voxjz` input or
    /// `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    #[command(flatten)]
    encoding_options: VoxjEncodingOptions,

    /// Emits the user-defined `ext` extension block. `--ext false` omits it.
    #[arg(
        value_name = "ext",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    ext: bool,

    /// When to record each object's build volume.
    #[arg(
        value_name = "edit-state",
        long,
        default_value = "auto",
        value_parser = cli_value_parser::<EditStateMode>()
    )]
    edit_state: EditStateMode,
}

impl VoxjOutput {
    /// The write format and output path for a document read from `input`.
    pub fn resolve(self, input: &Path) -> (WriteFormat, PathBuf) {
        let (serialization, mut options, output) =
            self.encoding_options.resolve_output(input, self.output);

        options.ext = self.ext;

        options.edit_state = self.edit_state;

        let format = WriteFormat::Voxj(VoxjWriteFormat {
            serialization,
            options,
        });

        (format, output)
    }
}
