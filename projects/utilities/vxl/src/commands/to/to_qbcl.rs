use crate::{Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the Qubicle format.
#[derive(Clone, Debug, Parser)]
#[command(name = "qbcl")]
pub struct ToQbcl {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.qbcl` file to write. Defaults to the input path with a
    /// `.qbcl` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,
}

impl ToQbcl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let output = self
            .output
            .unwrap_or_else(|| self.input.with_extension("qbcl"));
        dependencies.to_qbcl(&self.input, self.from, &output)
    }
}
