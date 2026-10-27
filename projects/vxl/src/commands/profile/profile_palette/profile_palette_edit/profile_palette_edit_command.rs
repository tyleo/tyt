use crate::{Dependencies, Result, commands::ProfilePaletteEditList};
use clap::Subcommand;

/// The `profile palette edit` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfilePaletteEditCommand {
    #[command(name = "list")]
    ProfilePaletteEditList(ProfilePaletteEditList),
}

impl ProfilePaletteEditCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfilePaletteEditCommand::ProfilePaletteEditList(list) => list.execute(dependencies),
        }
    }
}
