use crate::commands::CreateMse;
use clap::Subcommand;

/// Operations on material textures.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMaterial {
    CreateMse(CreateMse),
}

impl TytMaterial {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytMaterial::CreateMse(create_mse) => create_mse.execute(dependencies),
        }
    }
}
