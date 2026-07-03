use crate::commands::{PaletteList, PaletteShow};
use clap::Subcommand;

/// The `palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum PaletteCommand {
    #[command(name = "list")]
    PaletteList(PaletteList),
    #[command(name = "show")]
    PaletteShow(PaletteShow),
}

impl PaletteCommand {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            PaletteCommand::PaletteList(list) => list.execute(dependencies),
            PaletteCommand::PaletteShow(show) => show.execute(dependencies),
        }
    }
}
