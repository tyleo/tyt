use crate::{Dependencies, Result, commands::VoxDocCommand};
use clap::Parser;

/// Reports on, checks, and converts whole voxel documents.
#[derive(Clone, Debug, Parser)]
#[command(name = "vox-doc")]
pub struct VoxDoc {
    #[clap(subcommand)]
    pub command: VoxDocCommand,
}

impl VoxDoc {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
