use crate::{Dependencies, Result, commands::NodeCommand};
use clap::Parser;

/// Lists nodes or edits them into Voxel Json.
#[derive(Clone, Debug, Parser)]
#[command(name = "node")]
pub struct Node {
    #[clap(subcommand)]
    pub command: NodeCommand,
}

impl Node {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
