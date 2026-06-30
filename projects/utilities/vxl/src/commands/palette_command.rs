use crate::commands::Show;
use clap::Subcommand;

/// The `palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum PaletteCommand {
    #[command(name = "show")]
    Show(Show),
}

impl PaletteCommand {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            PaletteCommand::Show(show) => show.execute(_dependencies),
        }
    }
}
