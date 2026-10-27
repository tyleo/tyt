use crate::{Dependencies, Result, commands::ProfilePaletteShowCommand};
use clap::Parser;

/// Inspects the profiles `palette show --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct ProfilePaletteShow {
    #[clap(subcommand)]
    pub command: ProfilePaletteShowCommand,
}

impl ProfilePaletteShow {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
