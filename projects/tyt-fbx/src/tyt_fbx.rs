use crate::commands::{Extract, Hierarchy, Reduce, Rename};
use clap::Subcommand;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFbx {
    Extract(Extract),
    Hierarchy(Hierarchy),
    Reduce(Reduce),
    Rename(Rename),
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
