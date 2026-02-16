use clap::Subcommand;

use crate::commands::Extract;

/// Operations on FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TlFbx {
    Extract(Extract),
}

impl TlFbx {
    pub fn execute(self) {
        match self {
            TlFbx::Extract(extract) => {
                extract.execute();
            }
        }
    }
}
