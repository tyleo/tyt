use crate::{Dependencies, Result, commands::To};
use clap::Subcommand;

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Vxl {
    #[command(name = "to")]
    To(To),
}

impl Vxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            Vxl::To(to) => to.execute(dependencies),
        }
    }
}
