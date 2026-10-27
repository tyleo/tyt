use crate::{
    Dependencies, Result,
    commands::{VoxDocToGoxl, VoxDocToMvox, VoxDocToQbcl, VoxDocToVmax, VoxDocToVoxj},
};
use clap::Subcommand;

/// The `vox-doc to` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum VoxDocToCommand {
    #[command(name = "goxl")]
    VoxDocToGoxl(VoxDocToGoxl),

    #[command(name = "mvox")]
    VoxDocToMvox(VoxDocToMvox),

    #[command(name = "qbcl")]
    VoxDocToQbcl(VoxDocToQbcl),

    #[command(name = "vmax")]
    VoxDocToVmax(VoxDocToVmax),

    #[command(name = "voxj")]
    VoxDocToVoxj(VoxDocToVoxj),
}

impl VoxDocToCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            VoxDocToCommand::VoxDocToGoxl(goxl) => goxl.execute(dependencies),
            VoxDocToCommand::VoxDocToMvox(mvox) => mvox.execute(dependencies),
            VoxDocToCommand::VoxDocToQbcl(qbcl) => qbcl.execute(dependencies),
            VoxDocToCommand::VoxDocToVmax(vmax) => vmax.execute(dependencies),
            VoxDocToCommand::VoxDocToVoxj(voxj) => voxj.execute(dependencies),
        }
    }
}
