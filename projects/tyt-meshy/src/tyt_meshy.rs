use clap::Subcommand;

/// Commands for working with the Meshy API
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeshy {}

impl TytMeshy {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {}
    }
}
