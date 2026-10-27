use crate::{Dependencies, Result, commands::ProfileObjectVoxelsQuantizeCommand};
use clap::Parser;

/// Inspects the profiles `object voxels quantize --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "quantize")]
pub struct ProfileObjectVoxelsQuantize {
    #[clap(subcommand)]
    pub command: ProfileObjectVoxelsQuantizeCommand,
}

impl ProfileObjectVoxelsQuantize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
