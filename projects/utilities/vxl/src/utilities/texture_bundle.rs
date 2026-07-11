use crate::Texture;
use clap::ValueEnum;

/// A named `--texture` bundle: one value that expands to several single-map
/// presets, for a common material workflow.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TextureBundle {
    /// The glTF PBR set: albedo, orm, and emissive.
    #[value(name = "pbr")]
    Pbr,
}

impl TextureBundle {
    /// The single-map presets this bundle expands to, in order.
    pub fn textures(self) -> &'static [Texture] {
        match self {
            TextureBundle::Pbr => &[Texture::Albedo, Texture::Orm, Texture::Emissive],
        }
    }
}
