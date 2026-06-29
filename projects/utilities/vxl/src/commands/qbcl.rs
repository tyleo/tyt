use crate::{Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the Qubicle format.
#[derive(Clone, Debug, Parser)]
#[command(name = "qbcl")]
pub struct Qbcl {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.qbcl` file to write.
    #[arg(value_name = "output")]
    output: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,
}

impl Qbcl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.to_qbcl(&self.input, self.from, &self.output)
    }
}
