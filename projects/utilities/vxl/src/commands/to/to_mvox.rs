use crate::{Dependencies, Result, VoxelInput, commands::convert};
use clap::Parser;
use std::path::PathBuf;
use voxconv::WriteFormat;

/// Converts a voxel file to the MagicaVoxel format.
#[derive(Clone, Debug, Parser)]
#[command(name = "mvox")]
pub struct ToMvox {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.vox` file to write. Defaults to the input path with a
    /// `.vox` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,
}

impl ToMvox {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let to = WriteFormat::MVox;

        let output = self.input.output_path(self.output, to.extension());

        convert(&dependencies, &self.input.path, from, &output, &to)
    }
}
