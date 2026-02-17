use clap::Subcommand;

/// Operations on the filesystem
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFS {}

impl TytFS {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {}
    }
}
