use crate::{Dependencies, Result, commands::ObjectVoxelsCommand};
use clap::Parser;

/// Moves and quantizes objects' voxels within their grids, writing Voxel Json.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxels")]
pub struct ObjectVoxels {
    #[clap(subcommand)]
    pub command: ObjectVoxelsCommand,
}

impl ObjectVoxels {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
