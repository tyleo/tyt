use crate::commands::{
    EquirectToFaces, EquirectToNet, FacesToEquirect, FacesToPixelatedEquirect, PixelateFaces,
};
use clap::Subcommand;

/// Operations on cubemap images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytCubemap {
    FacesToEquirect(FacesToEquirect),
    FacesToPixelatedEquirect(FacesToPixelatedEquirect),
    EquirectToNet(EquirectToNet),
    EquirectToFaces(EquirectToFaces),
    PixelateFaces(PixelateFaces),
}

impl TytCubemap {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytCubemap::FacesToEquirect(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToPixelatedEquirect(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToFaces(cmd) => cmd.execute(dependencies),
            TytCubemap::PixelateFaces(cmd) => cmd.execute(dependencies),
        }
    }
}
