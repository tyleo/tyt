use crate::{Dependencies, Result, commands::ProfileSdfDocVoxelizeCommand};
use clap::Parser;

/// Inspects the profiles `sdf-doc voxelize --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxelize")]
pub struct ProfileSdfDocVoxelize {
    #[clap(subcommand)]
    pub command: ProfileSdfDocVoxelizeCommand,
}

impl ProfileSdfDocVoxelize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
