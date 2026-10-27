use crate::{Dependencies, Result, commands::NodeSetCommand};
use clap::Parser;

/// Sets a node property.
#[derive(Clone, Debug, Parser)]
#[command(name = "set")]
pub struct NodeSet {
    #[clap(subcommand)]
    pub command: NodeSetCommand,
}

impl NodeSet {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
