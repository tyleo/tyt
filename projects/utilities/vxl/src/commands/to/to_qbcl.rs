use crate::{Dependencies, ObjectSelection, Result, VoxelInput, commands::convert};
use clap::Parser;
use std::path::PathBuf;
use voxconv::WriteFormat;

/// Converts a voxel file to the Qubicle format.
#[derive(Clone, Debug, Parser)]
#[command(name = "qbcl")]
pub struct ToQbcl {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.qbcl` file to write. Defaults to the input path with a
    /// `.qbcl` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl ToQbcl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let to = WriteFormat::Qbcl;

        let output = self.input.output_path(self.output, to.extension());

        convert(
            &dependencies,
            &self.input.path,
            from,
            &output,
            &to,
            &self.selection,
        )
    }
}
