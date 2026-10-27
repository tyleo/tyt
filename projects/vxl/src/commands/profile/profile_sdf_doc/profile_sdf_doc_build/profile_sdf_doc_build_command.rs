use crate::{Dependencies, Result, commands::ProfileSdfDocBuildList};
use clap::Subcommand;

/// The `profile sdf-doc build` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileSdfDocBuildCommand {
    #[command(name = "list")]
    ProfileSdfDocBuildList(ProfileSdfDocBuildList),
}

impl ProfileSdfDocBuildCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileSdfDocBuildCommand::ProfileSdfDocBuildList(list) => list.execute(dependencies),
        }
    }
}
