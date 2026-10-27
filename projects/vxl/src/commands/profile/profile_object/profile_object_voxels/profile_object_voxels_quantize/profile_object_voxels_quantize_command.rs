use crate::{Dependencies, Result, commands::ProfileObjectVoxelsQuantizeList};
use clap::Subcommand;

/// The `profile object voxels quantize` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectVoxelsQuantizeCommand {
    #[command(name = "list")]
    ProfileObjectVoxelsQuantizeList(ProfileObjectVoxelsQuantizeList),
}

impl ProfileObjectVoxelsQuantizeCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectVoxelsQuantizeCommand::ProfileObjectVoxelsQuantizeList(list) => {
                list.execute(dependencies)
            }
        }
    }
}
