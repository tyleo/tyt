use crate::commands::{Mesh, Texture};
use clap::Subcommand;

/// Commands for working with the Meshy API
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeshy {
    #[command(name = "mesh")]
    Mesh(Mesh),
    #[command(name = "texture")]
    Texture(Texture),
}

impl TytMeshy {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytMeshy::Mesh(mesh) => mesh.execute(dependencies),
            TytMeshy::Texture(texture) => texture.execute(dependencies),
        }
    }
}
