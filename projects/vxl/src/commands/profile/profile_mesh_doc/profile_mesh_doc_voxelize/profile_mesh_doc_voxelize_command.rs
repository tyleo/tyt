use crate::{Dependencies, Result, commands::ProfileMeshDocVoxelizeList};
use clap::Subcommand;

/// The `profile mesh-doc voxelize` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileMeshDocVoxelizeCommand {
    #[command(name = "list")]
    ProfileMeshDocVoxelizeList(ProfileMeshDocVoxelizeList),
}

impl ProfileMeshDocVoxelizeCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileMeshDocVoxelizeCommand::ProfileMeshDocVoxelizeList(list) => {
                list.execute(dependencies)
            }
        }
    }
}
