use crate::commands::MeshOldTexture;
use clap::ValueEnum;

/// A named `--texture` bundle: one value that expands to several single-map
/// presets, for a common material workflow.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MeshOldTextureBundle {
    /// The glTF PBR set: albedo, orm, and emissive.
    #[value(name = "pbr")]
    Pbr,
}

impl MeshOldTextureBundle {
    /// The single-map presets this bundle expands to, in order.
    pub fn textures(self) -> &'static [MeshOldTexture] {
        match self {
            MeshOldTextureBundle::Pbr => &[
                MeshOldTexture::Albedo,
                MeshOldTexture::Orm,
                MeshOldTexture::Emissive,
            ],
        }
    }
}
