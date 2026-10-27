use crate::{
    Dependencies, Result,
    commands::{ProfilePaletteEdit, ProfilePaletteQuantize, ProfilePaletteShow},
};
use clap::Subcommand;

/// The `profile palette` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfilePaletteCommand {
    #[command(name = "edit")]
    ProfilePaletteEdit(ProfilePaletteEdit),

    #[command(name = "quantize")]
    ProfilePaletteQuantize(ProfilePaletteQuantize),

    #[command(name = "show")]
    ProfilePaletteShow(ProfilePaletteShow),
}

impl ProfilePaletteCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfilePaletteCommand::ProfilePaletteEdit(edit) => edit.execute(dependencies),

            ProfilePaletteCommand::ProfilePaletteQuantize(quantize) => {
                quantize.execute(dependencies)
            }

            ProfilePaletteCommand::ProfilePaletteShow(show) => show.execute(dependencies),
        }
    }
}
