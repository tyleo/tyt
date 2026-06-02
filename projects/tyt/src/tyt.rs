use crate::{Dependencies, Result};
use clap::Subcommand;
use tyt_claude::TytClaude;
use tyt_cubemap::TytCubemap;
use tyt_fbx::TytFbx;
use tyt_fs::TytFS;
use tyt_image::TytImage;
use tyt_material::TytMaterial;
use tyt_meta::TytMeta;
use tyt_oai::TytOAI;
use tyt_vmax::TytVMax;

/// The main command for `tyt`, which ties all my command-line tools together.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt {
    #[command(name = "claude")]
    Claude {
        #[clap(subcommand)]
        claude: TytClaude,
    },

    #[command(name = "cubemap")]
    Cubemap {
        #[clap(subcommand)]
        cubemap: TytCubemap,
    },

    #[command(name = "fs")]
    FS {
        #[clap(subcommand)]
        fs: TytFS,
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

    #[command(name = "oai")]
    OAI {
        #[clap(subcommand)]
        oai: TytOAI,
    },

    #[command(name = "vmax")]
    VMax {
        #[clap(subcommand)]
        vmax: TytVMax,
    },
}

impl Tyt {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        match self {
            Tyt::Claude { claude } => claude.execute(deps.tyt_claude_dependencies())?,
            Tyt::Cubemap { cubemap } => cubemap.execute(deps.tyt_cubemap_dependencies())?,
            Tyt::FS { fs } => fs.execute(deps.tyt_fs_dependencies())?,
            Tyt::Fbx { fbx } => fbx.execute(deps.tyt_fbx_dependencies())?,
            Tyt::Image { image } => image.execute(deps.tyt_image_dependencies())?,
            Tyt::Material { material } => material.execute(deps.tyt_material_dependencies())?,
            Tyt::Meta { meta } => meta.execute(deps.tyt_meta_dependencies())?,
            Tyt::OAI { oai } => oai.execute(deps.tyt_oai_dependencies())?,
            Tyt::VMax { vmax } => vmax.execute(deps.tyt_vmax_dependencies())?,
        }

        Ok(())
    }
}
