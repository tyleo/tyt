use crate::{Dependencies, Result, commands::IntegrationAgentsLinkCommand};
use clap::Parser;

/// Operations on the `.claude` link to `.agents`.
#[derive(Clone, Debug, Parser)]
#[command(name = "agents-link")]
pub struct IntegrationAgentsLink {
    #[clap(subcommand)]
    pub command: IntegrationAgentsLinkCommand,
}

impl IntegrationAgentsLink {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
