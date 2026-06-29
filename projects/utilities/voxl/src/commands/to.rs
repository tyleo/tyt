use crate::{Dependencies, Result, commands::ToCommand};
use clap::Parser;

/// Converts between voxel file formats.
#[derive(Clone, Debug, Parser)]
#[command(name = "to")]
pub struct To {
    #[clap(subcommand)]
    pub command: ToCommand,
}

impl To {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
