use crate::commands::{Hierarchy, Pack, RenameNode};
use clap::Subcommand;

/// Commands for working with Voxel Max.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytVMax {
    #[command(name = "hierarchy")]
    Hierarchy(Hierarchy),
    #[command(name = "pack")]
    Pack(Pack),
    #[command(name = "rename-node")]
    RenameNode(RenameNode),
}

impl TytVMax {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytVMax::Hierarchy(hierarchy) => hierarchy.execute(_dependencies),
            TytVMax::Pack(pack) => pack.execute(_dependencies),
            TytVMax::RenameNode(rename_node) => rename_node.execute(_dependencies),
        }
    }
}
