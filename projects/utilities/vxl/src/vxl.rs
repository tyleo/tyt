use crate::commands::Palette;
use crate::{Dependencies, Result, commands::To};
use clap::Subcommand;

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Vxl {
    #[command(name = "palette")]
    Palette(Palette),
    #[command(name = "to")]
    To(To),
}

impl Vxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            Vxl::Palette(palette) => palette.execute(dependencies),
            Vxl::To(to) => to.execute(dependencies),
        }
    }
}
