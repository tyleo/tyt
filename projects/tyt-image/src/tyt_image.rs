use crate::commands::{Pixelate, SquareImage};
use clap::Subcommand;

/// Operations on images.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytImage {
    #[command(name = "pixelate")]
    Pixelate(Pixelate),

    #[command(name = "square-image")]
    SquareImage(SquareImage),
}

impl TytImage {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytImage::Pixelate(cmd) => cmd.execute(dependencies),
            TytImage::SquareImage(cmd) => cmd.execute(dependencies),
        }
    }
}
