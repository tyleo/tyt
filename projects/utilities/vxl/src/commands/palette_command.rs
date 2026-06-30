use crate::commands::PaletteShow;
use clap::Subcommand;

/// The `palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum PaletteCommand {
    #[command(name = "show")]
    PaletteShow(PaletteShow),
}

impl PaletteCommand {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            PaletteCommand::PaletteShow(show) => show.execute(_dependencies),
        }
    }
}
