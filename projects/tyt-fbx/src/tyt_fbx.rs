use crate::commands;
use clap::Subcommand;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFbx {
    #[command(name = "extract")]
    Extract(commands::Extract),

    #[command(name = "hierarchy")]
    Hierarchy(commands::Hierarchy),

    #[command(name = "reduce")]
    Reduce(commands::Reduce),

    #[command(name = "rename")]
    Rename(commands::Rename),
}

impl TytFbx {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytFbx::Extract(extract) => extract.execute(dependencies),
            TytFbx::Hierarchy(hierarchy) => hierarchy.execute(dependencies),
            TytFbx::Reduce(reduce) => reduce.execute(dependencies),
            TytFbx::Rename(rename) => rename.execute(dependencies),
        }
    }
}
