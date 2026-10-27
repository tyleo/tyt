use crate::{Dependencies, Result, commands::ProfileSdfDocCommand};
use clap::Parser;

/// Inspects the profiles an `sdf-doc` command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "sdf-doc")]
pub struct ProfileSdfDoc {
    #[clap(subcommand)]
    pub command: ProfileSdfDocCommand,
}

impl ProfileSdfDoc {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
