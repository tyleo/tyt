use crate::commands::{Hierarchy, Pack, RenameNode, ToVoxelJson};
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
    #[command(name = "to-voxel-json")]
    ToVoxelJson(ToVoxelJson),
}

impl TytVMax {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytVMax::Hierarchy(hierarchy) => hierarchy.execute(dependencies),
            TytVMax::Pack(pack) => pack.execute(dependencies),
            TytVMax::RenameNode(rename_node) => rename_node.execute(dependencies),
            TytVMax::ToVoxelJson(to_voxel_json) => to_voxel_json.execute(dependencies),
        }
    }
}
