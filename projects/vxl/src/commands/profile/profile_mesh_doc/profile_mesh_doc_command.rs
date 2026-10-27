use crate::{Dependencies, Result, commands::ProfileMeshDocVoxelize};
use clap::Subcommand;

/// The `profile mesh-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileMeshDocCommand {
    #[command(name = "voxelize")]
    ProfileMeshDocVoxelize(ProfileMeshDocVoxelize),
}

impl ProfileMeshDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileMeshDocCommand::ProfileMeshDocVoxelize(voxelize) => {
                voxelize.execute(dependencies)
            }
        }
    }
}
