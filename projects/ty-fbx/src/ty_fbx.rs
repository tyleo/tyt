use clap::Subcommand;

use crate::commands::Extract;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TyFbx {
    Extract(Extract),
}

impl TyFbx {
    pub fn execute(self) {
        match self {
            TyFbx::Extract(extract) => {
                extract.execute();
            }
        }
    }
}
