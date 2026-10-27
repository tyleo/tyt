use crate::{Dependencies, Result, commands::ObjectCommand};
use clap::Parser;

/// Edits objects into Voxel Json, meshes them into glTF, or renders them to
/// the terminal or to PNG.
#[derive(Clone, Debug, Parser)]
#[command(name = "object")]
pub struct Object {
    #[clap(subcommand)]
    pub command: ObjectCommand,
}

impl Object {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
