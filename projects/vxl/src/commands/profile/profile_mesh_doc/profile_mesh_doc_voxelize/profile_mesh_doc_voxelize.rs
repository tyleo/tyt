use crate::{Dependencies, Result, commands::ProfileMeshDocVoxelizeCommand};
use clap::Parser;

/// Inspects the profiles `mesh-doc voxelize --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxelize")]
pub struct ProfileMeshDocVoxelize {
    #[clap(subcommand)]
    pub command: ProfileMeshDocVoxelizeCommand,
}

impl ProfileMeshDocVoxelize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
