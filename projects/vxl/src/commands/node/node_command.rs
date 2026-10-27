use crate::{
    Dependencies, Result,
    commands::{NodeAdd, NodeLink, NodeList, NodeRemove, NodeSet, NodeUnlink},
};
use clap::Subcommand;

/// The `node` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum NodeCommand {
    #[command(name = "add")]
    NodeAdd(NodeAdd),

    #[command(name = "link")]
    NodeLink(NodeLink),

    #[command(name = "list")]
    NodeList(NodeList),

    #[command(name = "remove")]
    NodeRemove(NodeRemove),

    #[command(name = "set")]
    NodeSet(NodeSet),

    #[command(name = "unlink")]
    NodeUnlink(NodeUnlink),
}

impl NodeCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            NodeCommand::NodeAdd(add) => add.execute(dependencies),
            NodeCommand::NodeLink(link) => link.execute(dependencies),
            NodeCommand::NodeList(list) => list.execute(dependencies),
            NodeCommand::NodeRemove(remove) => remove.execute(dependencies),
            NodeCommand::NodeSet(set) => set.execute(dependencies),
            NodeCommand::NodeUnlink(unlink) => unlink.execute(dependencies),
        }
    }
}
