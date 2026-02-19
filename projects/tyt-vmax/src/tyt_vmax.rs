use crate::commands::Hierarchy;
use clap::Subcommand;

/// Commands for working with Voxel Max.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytVMax {
    #[command(name = "hierarchy")]
    Hierarchy(Hierarchy),
}

impl TytVMax {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytVMax::Hierarchy(hierarchy) => hierarchy.execute(_dependencies),
        }
    }
}
