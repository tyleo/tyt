use crate::{Dependencies, Result, commands::ProfileObjectVoxelsQuantize};
use clap::Subcommand;

/// The `profile object voxels` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectVoxelsCommand {
    #[command(name = "quantize")]
    ProfileObjectVoxelsQuantize(ProfileObjectVoxelsQuantize),
}

impl ProfileObjectVoxelsCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectVoxelsCommand::ProfileObjectVoxelsQuantize(quantize) => {
                quantize.execute(dependencies)
            }
        }
    }
}
