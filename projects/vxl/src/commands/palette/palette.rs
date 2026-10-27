use crate::{Dependencies, Result, commands::PaletteCommand};
use clap::Parser;

/// Lists, shows, edits, and quantizes palettes.
#[derive(Clone, Debug, Parser)]
#[command(name = "palette")]
pub struct Palette {
    #[clap(subcommand)]
    pub command: PaletteCommand,
}

impl Palette {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
