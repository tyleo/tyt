use crate::{Dependencies, Result, commands::ProfilePaletteCommand};
use clap::Parser;

/// Inspects the profiles a `palette` command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "palette")]
pub struct ProfilePalette {
    #[clap(subcommand)]
    pub command: ProfilePaletteCommand,
}

impl ProfilePalette {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
