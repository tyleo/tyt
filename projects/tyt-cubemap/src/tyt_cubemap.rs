use crate::commands;
use clap::Subcommand;

/// Operations on cubemap images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytCubemap {
    #[command(name = "faces-to-equirect")]
    FacesToEquirect(commands::FacesToEquirect),

    #[command(name = "faces-to-pixelated-equirect")]
    FacesToPixelatedEquirect(commands::FacesToPixelatedEquirect),

    #[command(name = "equirect-to-net")]
    EquirectToNet(commands::EquirectToNet),

    #[command(name = "equirect-to-faces")]
    EquirectToFaces(commands::EquirectToFaces),

    #[command(name = "pixelate-faces")]
    PixelateFaces(commands::PixelateFaces),
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
