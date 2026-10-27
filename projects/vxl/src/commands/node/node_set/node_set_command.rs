use crate::{
    Dependencies, Result,
    commands::{NodeSetName, NodeSetPosition, NodeSetRotation, NodeSetScale},
};
use clap::Subcommand;

/// The `node set` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum NodeSetCommand {
    #[command(name = "name")]
    NodeSetName(NodeSetName),

    #[command(name = "position")]
    NodeSetPosition(NodeSetPosition),

    #[command(name = "rotation")]
    NodeSetRotation(NodeSetRotation),

    #[command(name = "scale")]
    NodeSetScale(NodeSetScale),
}

impl NodeSetCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            NodeSetCommand::NodeSetName(name) => name.execute(dependencies),
            NodeSetCommand::NodeSetPosition(position) => position.execute(dependencies),
            NodeSetCommand::NodeSetRotation(rotation) => rotation.execute(dependencies),
            NodeSetCommand::NodeSetScale(scale) => scale.execute(dependencies),
        }
    }
}
