use crate::{
    Dependencies, Result,
    commands::{ToGoxl, ToMvox, ToQbcl, ToVmax, ToVoxj},
};
use clap::Subcommand;

/// The `to` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ToCommand {
    #[command(name = "goxl")]
    ToGoxl(ToGoxl),
    #[command(name = "mvox")]
    ToMvox(ToMvox),
    #[command(name = "qbcl")]
    ToQbcl(ToQbcl),
    #[command(name = "vmax")]
    ToVmax(ToVmax),
    #[command(name = "voxj")]
    ToVoxj(ToVoxj),
}

impl ToCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ToCommand::ToGoxl(goxl) => goxl.execute(dependencies),
            ToCommand::ToMvox(mvox) => mvox.execute(dependencies),
            ToCommand::ToQbcl(qbcl) => qbcl.execute(dependencies),
            ToCommand::ToVmax(vmax) => vmax.execute(dependencies),
            ToCommand::ToVoxj(voxj) => voxj.execute(dependencies),
        }
    }
}
