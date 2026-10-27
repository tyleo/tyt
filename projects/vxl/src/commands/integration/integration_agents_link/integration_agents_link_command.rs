use crate::{
    Dependencies, Result,
    commands::{IntegrationAgentsLinkMove, IntegrationAgentsLinkNew},
};
use clap::Subcommand;

/// The `integration agents-link` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum IntegrationAgentsLinkCommand {
    #[command(name = "move")]
    IntegrationAgentsLinkMove(IntegrationAgentsLinkMove),

    #[command(name = "new")]
    IntegrationAgentsLinkNew(IntegrationAgentsLinkNew),
}

impl IntegrationAgentsLinkCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            IntegrationAgentsLinkCommand::IntegrationAgentsLinkMove(link_move) => {
                link_move.execute(dependencies)
            }

            IntegrationAgentsLinkCommand::IntegrationAgentsLinkNew(link_new) => {
                link_new.execute(dependencies)
            }
        }
    }
}
