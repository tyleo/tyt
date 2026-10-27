use crate::{Dependencies, Result, commands::ProfileObjectRenderList};
use clap::Subcommand;

/// The `profile object render` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectRenderCommand {
    #[command(name = "list")]
    ProfileObjectRenderList(ProfileObjectRenderList),
}

impl ProfileObjectRenderCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectRenderCommand::ProfileObjectRenderList(list) => list.execute(dependencies),
        }
    }
}
