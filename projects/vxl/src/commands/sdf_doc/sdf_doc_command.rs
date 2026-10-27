use crate::{
    Dependencies, Result,
    commands::{SdfDocBuild, SdfDocVoxelize},
};
use clap::Subcommand;

/// The `sdf-doc` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum SdfDocCommand {
    #[command(name = "build")]
    SdfDocBuild(SdfDocBuild),

    #[command(name = "voxelize")]
    SdfDocVoxelize(SdfDocVoxelize),
}

impl SdfDocCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            SdfDocCommand::SdfDocBuild(build) => build.execute(dependencies),
            SdfDocCommand::SdfDocVoxelize(voxelize) => voxelize.execute(dependencies),
        }
    }
}
