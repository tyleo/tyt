use crate::{
    Dependencies, Result,
    commands::{Goxl, Mvox, Qbcl, Vmax, Voxj},
};
use clap::Subcommand;

/// The `to` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ToCommand {
    #[command(name = "goxl")]
    Goxl(Goxl),
    #[command(name = "mvox")]
    Mvox(Mvox),
    #[command(name = "qbcl")]
    Qbcl(Qbcl),
    #[command(name = "vmax")]
    Vmax(Vmax),
    #[command(name = "voxj")]
    Voxj(Voxj),
}

impl ToCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ToCommand::Goxl(goxl) => goxl.execute(dependencies),
            ToCommand::Mvox(mvox) => mvox.execute(dependencies),
            ToCommand::Qbcl(qbcl) => qbcl.execute(dependencies),
            ToCommand::Vmax(vmax) => vmax.execute(dependencies),
            ToCommand::Voxj(voxj) => voxj.execute(dependencies),
        }
    }
}
