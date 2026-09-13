use crate::commands::{MeshOldTexture, MeshOldTextureBundle};
use clap::ValueEnum;

/// A `--texture` value: a single-map [`MeshOldTexture`] preset or a [`MeshOldTextureBundle`]
/// that expands to several. One flat enum so clap validates and completes both.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MeshOldTextureArg {
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

impl MeshOldTextureArg {
    /// The single-map presets this value bakes, in order.
    pub fn textures(self) -> Vec<MeshOldTexture> {
        match self {
            MeshOldTextureArg::Albedo => vec![MeshOldTexture::Albedo],
            MeshOldTextureArg::Orm => vec![MeshOldTexture::Orm],
            MeshOldTextureArg::MetallicRoughness => vec![MeshOldTexture::MetallicRoughness],
            MeshOldTextureArg::MetallicSmoothness => vec![MeshOldTexture::MetallicSmoothness],
            MeshOldTextureArg::Mse => vec![MeshOldTexture::Mse],
            MeshOldTextureArg::Emissive => vec![MeshOldTexture::Emissive],
            MeshOldTextureArg::Occlusion => vec![MeshOldTexture::Occlusion],
            MeshOldTextureArg::ComputedOcclusion => vec![MeshOldTexture::ComputedOcclusion],
            MeshOldTextureArg::Roughness => vec![MeshOldTexture::Roughness],
            MeshOldTextureArg::Smoothness => vec![MeshOldTexture::Smoothness],
            MeshOldTextureArg::Pbr => MeshOldTextureBundle::Pbr.textures().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{MeshOldTexture, MeshOldTextureArg};
    use clap::ValueEnum;

    #[test]
    fn the_pbr_bundle_expands_to_albedo_orm_and_emissive() {
        assert_eq!(
            MeshOldTextureArg::Pbr.textures(),
            vec![
                MeshOldTexture::Albedo,
                MeshOldTexture::Orm,
                MeshOldTexture::Emissive
            ]
        );
    }

    #[test]
    fn every_single_preset_is_a_texture_arg_of_the_same_name() {
        // Guards the two lists against drift: every preset must round-trip
        // through `MeshOldTextureArg` as itself.
        for texture in MeshOldTexture::value_variants() {
            let name = texture.cli_name();
            let arg = MeshOldTextureArg::from_str(&name, true)
                .unwrap_or_else(|_| panic!("`{name}` is missing from MeshOldTextureArg"));
            assert_eq!(
                arg.textures(),
                vec![*texture],
                "`{name}` must expand to just itself"
            );
        }
    }
}
