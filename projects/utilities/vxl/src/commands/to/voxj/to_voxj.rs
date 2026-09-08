use crate::{
    Dependencies, ObjectSelection, Result, VoxelInput, VoxjEncodingOptions, cli_value_parser,
    commands::convert,
};
use clap::{ArgAction, Parser};
use std::path::PathBuf;
use voxconv::{WriteFormat, voxj::EditStateMode};

/// Converts a voxel file to the Voxel JSON format.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxj")]
pub struct ToVoxj {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.voxj` or `.voxjz` document to write. Defaults to the input
    /// path with a `.voxj` extension, or `.voxjz` when `--format zip`.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

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
    #[arg(
        value_name = "edit-state",
        long,
        default_value = "auto",
        value_parser = cli_value_parser::<EditStateMode>()
    )]
    edit_state: EditStateMode,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let (serialization, mut options, output) = self
            .encoding_options
            .resolve_output(&self.input.path, self.output);

        options.ext = self.ext;

        options.edit_state = self.edit_state;

        convert(
            &dependencies,
            &self.input.path,
            from,
            &output,
            &WriteFormat::Voxj {
                serialization,
                options,
            },
            &self.selection,
        )
    }
}
