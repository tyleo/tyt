use crate::{Dependencies, Result, commands::HierarchyCommand};
use clap::Parser;

/// Commands for working with the scene-graph.
#[derive(Clone, Debug, Parser)]
#[command(name = "hierarchy")]
pub struct Hierarchy {
    #[clap(subcommand)]
    pub command: HierarchyCommand,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
