use crate::{Dependencies, Result, commands::IntegrationSkillCommand};
use clap::Parser;

/// Agent skill operations.
#[derive(Clone, Debug, Parser)]
#[command(name = "skill")]
pub struct IntegrationSkill {
    #[clap(subcommand)]
    pub command: IntegrationSkillCommand,
}

impl IntegrationSkill {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        self.command.execute(dependencies)
    }
}
