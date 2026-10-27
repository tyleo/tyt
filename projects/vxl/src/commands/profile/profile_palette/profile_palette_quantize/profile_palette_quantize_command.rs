use crate::{Dependencies, Result, commands::ProfilePaletteQuantizeList};
use clap::Subcommand;

/// The `profile palette quantize` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfilePaletteQuantizeCommand {
    #[command(name = "list")]
    ProfilePaletteQuantizeList(ProfilePaletteQuantizeList),
}

impl ProfilePaletteQuantizeCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfilePaletteQuantizeCommand::ProfilePaletteQuantizeList(list) => {
                list.execute(dependencies)
            }
        }
    }
}
