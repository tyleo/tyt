use crate::commands::Find;
use clap::Subcommand;

/// Operations on the filesystem
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFS {
    #[command(name = "find")]
    Find(Find),
}

impl TytFS {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytFS::Find(find) => find.execute(_dependencies),
        }
    }
}
