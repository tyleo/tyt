use crate::{Dependencies, Result, commands::VoxDocToCommand};
use clap::Parser;

/// Converts between voxel file formats.
#[derive(Clone, Debug, Parser)]
#[command(name = "to")]
pub struct VoxDocTo {
    #[clap(subcommand)]
    pub command: VoxDocToCommand,
}

impl VoxDocTo {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
