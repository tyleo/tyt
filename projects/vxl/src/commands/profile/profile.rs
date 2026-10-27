use crate::{Dependencies, Result, commands::ProfileCommand};
use clap::Parser;

/// Inspects the profiles a command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "profile")]
pub struct Profile {
    #[clap(subcommand)]
    pub command: ProfileCommand,
}

impl Profile {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
