use crate::commands::Mesh;
use clap::Subcommand;

/// Commands for working with the Meshy API
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeshy {
    #[command(name = "mesh")]
    Mesh(Mesh),
}

impl TytMeshy {
    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytMeshy::Mesh(mesh) => mesh.execute(dependencies),
        }
    }
}
