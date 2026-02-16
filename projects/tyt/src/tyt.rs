use crate::{Dependencies, Result};
use clap::Subcommand;
use tyt_cubemap::TytCubemap;
use tyt_fbx::TytFbx;
use tyt_image::TytImage;
use tyt_material::TytMaterial;
use tyt_meta::TytMeta;

/// The main command for `tyt`, which ties all my command-line tools together.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt {
    #[command(name = "cubemap")]
    Cubemap {
        #[clap(subcommand)]
        cubemap: TytCubemap,
    },

    #[command(name = "fbx")]
    Fbx {
        #[clap(subcommand)]
        fbx: TytFbx,
    },

    #[command(name = "image")]
    Image {
        #[clap(subcommand)]
        image: TytImage,
    },

    #[command(name = "material")]
    Material {
        #[clap(subcommand)]
        material: TytMaterial,
    },

    #[command(name = "meta")]
    Meta {
        #[clap(subcommand)]
        meta: TytMeta,
    },
}

impl Tyt {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        match self {
            Tyt::Cubemap { cubemap } => cubemap.execute(deps.tyt_cubemap_dependencies())?,
            Tyt::Fbx { fbx } => fbx.execute(deps.tyt_fbx_dependencies())?,
            Tyt::Image { image } => image.execute(deps.tyt_image_dependencies())?,
            Tyt::Material { material } => material.execute(deps.tyt_material_dependencies())?,
            Tyt::Meta { meta } => meta.execute(deps.tyt_meta_dependencies())?,
        }

        Ok(())
    }
}
