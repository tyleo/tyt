use crate::{Dependencies, Result, VoxelInput, commands::convert};
use clap::Parser;
use std::path::PathBuf;
use voxconv::WriteFormat;

/// Converts a voxel file to the Goxel format.
#[derive(Clone, Debug, Parser)]
#[command(name = "goxl")]
pub struct ToGoxl {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.gox` file to write. Defaults to the input path with a
    /// `.gox` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,
}

impl ToGoxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let to = WriteFormat::Goxl;

        let output = self.input.output_path(self.output, to.extension());

        convert(&dependencies, &self.input.path, from, &output, &to)
    }
}
