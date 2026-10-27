use crate::{Dependencies, Result, commands::ProfileSdfDocBuildCommand};
use clap::Parser;

/// Inspects the profiles `sdf-doc build --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "build")]
pub struct ProfileSdfDocBuild {
    #[clap(subcommand)]
    pub command: ProfileSdfDocBuildCommand,
}

impl ProfileSdfDocBuild {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
