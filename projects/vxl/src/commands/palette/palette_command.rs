use crate::{
    Dependencies, Result,
    commands::{PaletteEdit, PaletteList, PaletteQuantize, PaletteShow},
};
use clap::Subcommand;

/// The `palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum PaletteCommand {
    #[command(name = "edit")]
    PaletteEdit(PaletteEdit),

    #[command(name = "list")]
    PaletteList(PaletteList),

    #[command(name = "quantize")]
    PaletteQuantize(PaletteQuantize),

    #[command(name = "show")]
    PaletteShow(PaletteShow),
}

impl PaletteCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            PaletteCommand::PaletteEdit(edit) => edit.execute(dependencies),
            PaletteCommand::PaletteList(list) => list.execute(dependencies),
            PaletteCommand::PaletteQuantize(quantize) => quantize.execute(dependencies),
            PaletteCommand::PaletteShow(show) => show.execute(dependencies),
        }
    }
}
