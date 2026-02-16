use crate::commands::{
    EquirectToFaces, EquirectToNet, FacesToEquirect, FacesToPixelatedEquirect, PixelateFaces,
};
use clap::Subcommand;

/// Operations on cubemap images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytCubemap {
    #[command(name = "faces-to-equirect")]
    FacesToEquirect(FacesToEquirect),

    #[command(name = "faces-to-pixelated-equirect")]
    FacesToPixelatedEquirect(FacesToPixelatedEquirect),

    #[command(name = "equirect-to-net")]
    EquirectToNet(EquirectToNet),

    #[command(name = "equirect-to-faces")]
    EquirectToFaces(EquirectToFaces),

    #[command(name = "pixelate-faces")]
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
