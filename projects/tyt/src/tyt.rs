use crate::{Dependencies, Result};
use clap::Subcommand;
use tyt_fbx::TytFbx;

/// The main command for `tyt`, which ties all my command-line tools together.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt {
    Fbx {
        #[clap(subcommand)]
        fbx: TytFbx,
    },
}

impl Tyt {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        match self {
            Tyt::Fbx { fbx } => fbx.execute(deps.tyt_fbx_dependencies())?,
        }

        Ok(())
    }
}
