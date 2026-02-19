use crate::commands::{Find, MoveToScratch};
use clap::Subcommand;

/// Operations on the filesystem
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFS {
    #[command(name = "find")]
    Find(Find),
    #[command(name = "move-to-scratch")]
    MoveToScratch(MoveToScratch),
}

impl TytFS {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytFS::Find(find) => find.execute(dependencies),
            TytFS::MoveToScratch(move_to_scratch) => move_to_scratch.execute(dependencies),
        }
    }
}
