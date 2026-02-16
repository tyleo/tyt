use crate::commands::Extract;
use clap::Subcommand;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TyFbx {
    Extract(Extract),
}

impl TyFbx {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TyFbx::Extract(extract) => extract.execute(dependencies),
        }
    }
}
