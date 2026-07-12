use crate::{Dependencies, Result, commands::PaletteCommand};
use clap::Parser;

/// Palette operations.
#[derive(Clone, Debug, Parser)]
#[command(name = "palette")]
pub struct Palette {
    #[clap(subcommand)]
    pub command: PaletteCommand,
}

impl Palette {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
