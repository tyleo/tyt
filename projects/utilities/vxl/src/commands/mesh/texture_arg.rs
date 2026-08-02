use crate::commands::{Texture, TextureBundle};
use clap::ValueEnum;

/// A `--texture` value: a single-map [`Texture`] preset or a [`TextureBundle`]
/// that expands to several. One flat enum so clap validates and completes both.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TextureArg {
    /// RGBA base color from `baseColor`. Four channels.
    #[value(name = "albedo")]
    Albedo,

    /// glTF occlusion-roughness-metallic: R = occlusion, G = roughness,
    /// B = metallic. Three channels.
    #[value(name = "orm")]
    Orm,

    /// glTF metallic-roughness: R = 0, G = roughness, B = metallic. Three
    /// channels.
    #[value(name = "metallic-roughness")]
    MetallicRoughness,

    /// Unity metallic-smoothness: R = metallic, A = smoothness, G and B = 0.
    /// Four channels.
    #[value(name = "metallic-smoothness")]
    MetallicSmoothness,

    /// MSE packing: R = metallic, G = smoothness, B = emissive. Three channels.
    #[value(name = "mse")]
    Mse,

    /// The emissive color: `emissiveColor` scaled by `emissiveStrength`, so the
    /// glTF emissive slot glows in the surface's own emissive color.
    #[value(name = "emissive")]
    Emissive,

    /// Grayscale `occlusionStrength`. One channel.
    #[value(name = "occlusion")]
    Occlusion,

    /// Grayscale occlusion computed from the voxel geometry. One channel; always
    /// an unwrap layout, not yet supported, so hidden until it ships.
    #[value(name = "computed-occlusion", hide = true)]
    ComputedOcclusion,

    /// Grayscale `roughness`. One channel.
    #[value(name = "roughness")]
    Roughness,

    /// Grayscale `smoothness`, the derived `1-roughness`. One channel.
    #[value(name = "smoothness")]
    Smoothness,

    /// The glTF PBR set: albedo, orm, and emissive.
    #[value(name = "pbr")]
    Pbr,
}

impl TextureArg {
    /// The single-map presets this value bakes, in order.
    pub fn textures(self) -> Vec<Texture> {
        match self {
            TextureArg::Albedo => vec![Texture::Albedo],
            TextureArg::Orm => vec![Texture::Orm],
            TextureArg::MetallicRoughness => vec![Texture::MetallicRoughness],
            TextureArg::MetallicSmoothness => vec![Texture::MetallicSmoothness],
            TextureArg::Mse => vec![Texture::Mse],
            TextureArg::Emissive => vec![Texture::Emissive],
            TextureArg::Occlusion => vec![Texture::Occlusion],
            TextureArg::ComputedOcclusion => vec![Texture::ComputedOcclusion],
            TextureArg::Roughness => vec![Texture::Roughness],
            TextureArg::Smoothness => vec![Texture::Smoothness],
            TextureArg::Pbr => TextureBundle::Pbr.textures().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{Texture, TextureArg};
    use clap::ValueEnum;

    #[test]
    fn the_pbr_bundle_expands_to_albedo_orm_and_emissive() {
        assert_eq!(
            TextureArg::Pbr.textures(),
            vec![Texture::Albedo, Texture::Orm, Texture::Emissive]
        );
    }

    #[test]
    fn every_single_preset_is_a_texture_arg_of_the_same_name() {
        // Guards the two lists against drift: every preset must round-trip
        // through `TextureArg` as itself.
        for texture in Texture::value_variants() {
            let name = texture.cli_name();
            let arg = TextureArg::from_str(&name, true)
                .unwrap_or_else(|_| panic!("`{name}` is missing from TextureArg"));
            assert_eq!(
                arg.textures(),
                vec![*texture],
                "`{name}` must expand to just itself"
            );
        }
    }
}
