use crate::{Dependencies, Result, commands::ProfilePaletteEditCommand};
use clap::Parser;

/// Inspects the profiles `palette edit --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "edit")]
pub struct ProfilePaletteEdit {
    #[clap(subcommand)]
    pub command: ProfilePaletteEditCommand,
}

impl ProfilePaletteEdit {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
