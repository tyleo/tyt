use crate::commands::CreateCommand;
use clap::Subcommand;

/// Meta-tools for scaffolding new tyt sub-crates and commands.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeta {
    #[command(name = "create-command")]
    CreateCommand(CreateCommand),
}

impl TytMeta {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytMeta::CreateCommand(cmd) => cmd.execute(dependencies),
        }
    }
}
