use crate::{Dependencies, Result, commands::ObjectSetCommand};
use clap::Parser;

/// Sets an object property.
#[derive(Clone, Debug, Parser)]
#[command(name = "set")]
pub struct ObjectSet {
    #[clap(subcommand)]
    pub command: ObjectSetCommand,
}

impl ObjectSet {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
