use crate::commands::{Extract, Reduce, Rename};
use clap::Subcommand;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TyFbx {
    Extract(Extract),
    Reduce(Reduce),
    Rename(Rename),
}

impl TyFbx {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TyFbx::Extract(extract) => extract.execute(dependencies),
            TyFbx::Reduce(reduce) => reduce.execute(dependencies),
            TyFbx::Rename(rename) => rename.execute(dependencies),
        }
    }
}
