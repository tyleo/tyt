use crate::commands;
use clap::Subcommand;

/// Operations on cubemap images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytCubemap {
    #[command(name = "c6x1-to-equirect")]
    C6x1ToEquirect(commands::C6x1ToEquirect),

    #[command(name = "c6x1-to-faces")]
    C6x1ToFaces(commands::C6x1ToFaces),

    #[command(name = "c6x1-to-net")]
    C6x1ToNet(commands::C6x1ToNet),

    #[command(name = "equirect-to-c6x1")]
    EquirectToC6x1(commands::EquirectToC6x1),

    #[command(name = "equirect-to-faces")]
    EquirectToFaces(commands::EquirectToFaces),

    #[command(name = "equirect-to-net")]
    EquirectToNet(commands::EquirectToNet),

    #[command(name = "faces-to-c6x1")]
    FacesToC6x1(commands::FacesToC6x1),

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
            TytCubemap::C6x1ToEquirect(cmd) => cmd.execute(dependencies),
            TytCubemap::C6x1ToFaces(cmd) => cmd.execute(dependencies),
            TytCubemap::C6x1ToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToC6x1(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToFaces(cmd) => cmd.execute(dependencies),
            TytCubemap::EquirectToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToC6x1(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToEquirect(cmd) => cmd.execute(dependencies),
            TytCubemap::FacesToNet(cmd) => cmd.execute(dependencies),
            TytCubemap::PixelateFaces(cmd) => cmd.execute(dependencies),
        }
    }
}
