use crate::{Dependencies, Result, commands::ProfileObjectCommand};
use clap::Parser;

/// Inspects the profiles an `object` command can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "object")]
pub struct ProfileObject {
    #[clap(subcommand)]
    pub command: ProfileObjectCommand,
}

impl ProfileObject {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
