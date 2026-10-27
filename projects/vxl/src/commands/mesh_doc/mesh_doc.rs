use crate::{Dependencies, Result, commands::MeshDocCommand};
use clap::Parser;

/// Works with whole mesh documents.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh-doc")]
pub struct MeshDoc {
    #[clap(subcommand)]
    pub command: MeshDocCommand,
}

impl MeshDoc {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
