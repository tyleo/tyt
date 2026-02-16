use crate::{Dependencies, Result};
use clap::Subcommand;
use tyt_cubemap::TytCubemap;
use tyt_fbx::TytFbx;
use tyt_material::TytMaterial;

/// The main command for `tyt`, which ties all my command-line tools together.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt {
    Cubemap {
        #[clap(subcommand)]
        cubemap: TytCubemap,
    },
    Fbx {
        #[clap(subcommand)]
        fbx: TytFbx,
    },
    Material {
        #[clap(subcommand)]
        material: TytMaterial,
    },
}

impl Tyt {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        match self {
            Tyt::Cubemap { cubemap } => cubemap.execute(deps.tyt_cubemap_dependencies())?,
            Tyt::Fbx { fbx } => fbx.execute(deps.tyt_fbx_dependencies())?,
            Tyt::Material { material } => material.execute(deps.tyt_material_dependencies())?,
        }

        Ok(())
    }
}
