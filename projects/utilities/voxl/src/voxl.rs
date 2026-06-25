use clap::Subcommand;

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Voxl {}

impl Voxl {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {}
    }
}
