use crate::{Dependencies, Result, commands::ProfileSdfDocVoxelizeList};
use clap::Subcommand;

/// The `profile sdf-doc voxelize` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileSdfDocVoxelizeCommand {
    #[command(name = "list")]
    ProfileSdfDocVoxelizeList(ProfileSdfDocVoxelizeList),
}

impl ProfileSdfDocVoxelizeCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileSdfDocVoxelizeCommand::ProfileSdfDocVoxelizeList(list) => {
                list.execute(dependencies)
            }
        }
    }
}
