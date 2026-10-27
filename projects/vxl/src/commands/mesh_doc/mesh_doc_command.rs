use crate::{Dependencies, Result, commands::MeshDocVoxelize};
use clap::Subcommand;

/// The `mesh-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum MeshDocCommand {
    #[command(name = "voxelize")]
    MeshDocVoxelize(MeshDocVoxelize),
}

impl MeshDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            MeshDocCommand::MeshDocVoxelize(voxelize) => voxelize.execute(dependencies),
        }
    }
}
