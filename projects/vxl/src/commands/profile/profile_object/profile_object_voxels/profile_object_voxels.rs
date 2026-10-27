use crate::{Dependencies, Result, commands::ProfileObjectVoxelsCommand};
use clap::Parser;

/// Inspects the profiles an `object voxels` command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxels")]
pub struct ProfileObjectVoxels {
    #[clap(subcommand)]
    pub command: ProfileObjectVoxelsCommand,
}

impl ProfileObjectVoxels {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
