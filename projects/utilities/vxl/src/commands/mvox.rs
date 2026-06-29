use crate::{Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the MagicaVoxel format.
#[derive(Clone, Debug, Parser)]
#[command(name = "mvox")]
pub struct Mvox {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.vox` file to write. Defaults to the input path with a
    /// `.vox` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,
}

impl Mvox {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension("vox"));
        dependencies.to_mvox(&self.input, self.from, &output)
    }
}
