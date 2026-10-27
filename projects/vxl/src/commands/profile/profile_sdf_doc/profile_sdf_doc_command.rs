use crate::{
    Dependencies, Result,
    commands::{ProfileSdfDocBuild, ProfileSdfDocVoxelize},
};
use clap::Subcommand;

/// The `profile sdf-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileSdfDocCommand {
    #[command(name = "build")]
    ProfileSdfDocBuild(ProfileSdfDocBuild),

    #[command(name = "voxelize")]
    ProfileSdfDocVoxelize(ProfileSdfDocVoxelize),
}

impl ProfileSdfDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileSdfDocCommand::ProfileSdfDocBuild(build) => build.execute(dependencies),
            ProfileSdfDocCommand::ProfileSdfDocVoxelize(voxelize) => voxelize.execute(dependencies),
        }
    }
}
