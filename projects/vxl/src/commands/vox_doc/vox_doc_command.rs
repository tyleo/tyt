use crate::{
    Dependencies, Result,
    commands::{VoxDocShow, VoxDocTo, VoxDocValidate},
};
use clap::Subcommand;

/// The `vox-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum VoxDocCommand {
    #[command(name = "show")]
    VoxDocShow(VoxDocShow),

    #[command(name = "to")]
    VoxDocTo(VoxDocTo),

    #[command(name = "validate")]
    VoxDocValidate(VoxDocValidate),
}

impl VoxDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            VoxDocCommand::VoxDocShow(show) => show.execute(dependencies),
            VoxDocCommand::VoxDocTo(to) => to.execute(dependencies),
            VoxDocCommand::VoxDocValidate(validate) => validate.execute(dependencies),
        }
    }
}
