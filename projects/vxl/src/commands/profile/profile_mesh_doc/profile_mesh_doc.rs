use crate::{Dependencies, Result, commands::ProfileMeshDocCommand};
use clap::Parser;

/// Inspects the profiles a `mesh-doc` command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh-doc")]
pub struct ProfileMeshDoc {
    #[clap(subcommand)]
    pub command: ProfileMeshDocCommand,
}

impl ProfileMeshDoc {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
