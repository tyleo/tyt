use crate::{Dependencies, Result, commands::ProfilePaletteQuantizeCommand};
use clap::Parser;

/// Inspects the profiles `palette quantize --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "quantize")]
pub struct ProfilePaletteQuantize {
    #[clap(subcommand)]
    pub command: ProfilePaletteQuantizeCommand,
}

impl ProfilePaletteQuantize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
