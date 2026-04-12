use crate::commands;
use clap::Subcommand;

/// Operations on cubemap images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytCubemap {
    #[command(name = "equirect-to-6x1")]
    EquirectTo6x1(commands::EquirectTo6x1),

    #[command(name = "equirect-to-faces")]
    EquirectToFaces(commands::EquirectToFaces),

    #[command(name = "equirect-to-net")]
    EquirectToNet(commands::EquirectToNet),

    #[command(name = "faces-to-6x1")]
    FacesTo6x1(commands::FacesTo6x1),

    #[command(name = "faces-to-equirect")]
    FacesToEquirect(commands::FacesToEquirect),

    #[command(name = "faces-to-net")]
    FacesToNet(commands::FacesToNet),

    #[command(name = "pixelate-faces")]
    PixelateFaces(commands::PixelateFaces),
}

impl TytCubemap {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytCubemap::EquirectTo6x1(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToFaces(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesTo6x1(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToEquirect(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::PixelateFaces(cmd) => cmd.execute(dependencies),
        }
    }
}
