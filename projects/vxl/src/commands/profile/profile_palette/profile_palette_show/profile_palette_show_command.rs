use crate::{Dependencies, Result, commands::ProfilePaletteShowList};
use clap::Subcommand;

/// The `profile palette show` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfilePaletteShowCommand {
    #[command(name = "list")]
    ProfilePaletteShowList(ProfilePaletteShowList),
}

impl ProfilePaletteShowCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfilePaletteShowCommand::ProfilePaletteShowList(list) => list.execute(dependencies),
        }
    }
}
