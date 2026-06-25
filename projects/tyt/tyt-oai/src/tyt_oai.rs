use crate::commands::Img;
use clap::Subcommand;

/// Commands for working with the OpenAI API.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytOAI {
    #[command(name = "img")]
    Img(Img),
}

impl TytOAI {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytOAI::Img(img) => img.execute(dependencies),
        }
    }
}
