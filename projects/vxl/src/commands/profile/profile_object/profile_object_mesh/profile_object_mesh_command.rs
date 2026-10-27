use crate::{Dependencies, Result, commands::ProfileObjectMeshList};
use clap::Subcommand;

/// The `profile object mesh` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectMeshCommand {
    #[command(name = "list")]
    ProfileObjectMeshList(ProfileObjectMeshList),
}

impl ProfileObjectMeshCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectMeshCommand::ProfileObjectMeshList(list) => list.execute(dependencies),
        }
    }
}
