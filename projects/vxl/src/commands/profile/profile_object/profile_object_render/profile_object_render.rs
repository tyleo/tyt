use crate::{Dependencies, Result, commands::ProfileObjectRenderCommand};
use clap::Parser;

/// Inspects the profiles `object render --profile` can apply.
#[derive(Clone, Debug, Parser)]
#[command(name = "render")]
pub struct ProfileObjectRender {
    #[clap(subcommand)]
    pub command: ProfileObjectRenderCommand,
}

impl ProfileObjectRender {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
