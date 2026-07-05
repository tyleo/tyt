use crate::{ChannelPacking, ChannelSource, TextureBake};
use clap::ValueEnum;
use voxsmith::{EMISSIVE_STRENGTH, METALLIC_FACTOR, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR};

/// A named `--texture` preset, a common material-map packing.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Texture {
    /// RGBA base color from `baseColorFactor`. Four channels.
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

    /// The emissive color: `emissiveFactor` scaled by `emissiveStrength`, so the
    /// glTF emissive slot glows in the surface's own emissive color.
    #[value(name = "emissive")]
    Emissive,

    /// Grayscale `occlusionStrength`. One channel.
    #[value(name = "occlusion")]
    Occlusion,

    /// Grayscale occlusion computed from the voxel geometry. One channel; always
    /// an unwrap layout.
    #[value(name = "computed-occlusion")]
    ComputedOcclusion,

    /// Grayscale `roughnessFactor`. One channel.
    #[value(name = "roughness")]
    Roughness,

    /// Grayscale `smoothness`, the derived `1-roughnessFactor`. One channel.
    #[value(name = "smoothness")]
    Smoothness,
}

impl Texture {
    /// Lowers this preset to the map the bake writes.
    pub fn bake(self) -> TextureBake {
        match self {
            Texture::Albedo => TextureBake::RgbaColor,

            Texture::Orm => packing(
                attribute(OCCLUSION_STRENGTH, false),
                attribute(ROUGHNESS_FACTOR, false),
                attribute(METALLIC_FACTOR, false),
                None,
            ),

            Texture::MetallicRoughness => packing(
                Some(ChannelSource::Zero),
                attribute(ROUGHNESS_FACTOR, false),
                attribute(METALLIC_FACTOR, false),
                None,
            ),

            Texture::MetallicSmoothness => packing(
                attribute(METALLIC_FACTOR, false),
                Some(ChannelSource::Zero),
                Some(ChannelSource::Zero),
                attribute(ROUGHNESS_FACTOR, true),
            ),

            Texture::Mse => packing(
                attribute(METALLIC_FACTOR, false),
                attribute(ROUGHNESS_FACTOR, true),
                attribute(EMISSIVE_STRENGTH, false),
                None,
            ),

            Texture::Emissive => TextureBake::EmissiveColor,

            Texture::Occlusion => packing(attribute(OCCLUSION_STRENGTH, false), None, None, None),

            Texture::ComputedOcclusion => {
                packing(Some(ChannelSource::ComputedOcclusion), None, None, None)
            }

            Texture::Roughness => packing(attribute(ROUGHNESS_FACTOR, false), None, None, None),

            Texture::Smoothness => packing(attribute(ROUGHNESS_FACTOR, true), None, None, None),
        }
    }
}

/// An attribute channel source by voxj key, inverted when `invert` is set.
fn attribute(key: &str, invert: bool) -> Option<ChannelSource> {
    Some(ChannelSource::Attribute {
        key: key.to_string(),
        component: None,
        invert,
    })
}

/// A `TextureBake::Packing` from per-channel sources.
fn packing(
    r: Option<ChannelSource>,
    g: Option<ChannelSource>,
    b: Option<ChannelSource>,
    a: Option<ChannelSource>,
) -> TextureBake {
    TextureBake::Packing(ChannelPacking::new(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use crate::{ChannelPacking, ChannelSource, Texture, TextureBake};
    use voxsmith::{METALLIC_FACTOR, ROUGHNESS_FACTOR};

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    #[test]
    fn albedo_is_the_rgba_color() {
        assert_eq!(Texture::Albedo.bake(), TextureBake::RgbaColor);
    }

    #[test]
    fn mse_matches_its_texture_map_equivalent() {
        let manual = "R=metallicFactor,G=smoothness,B=emissiveStrength"
            .parse::<ChannelPacking>()
            .unwrap();
        assert_eq!(Texture::Mse.bake(), TextureBake::Packing(manual));
    }

    #[test]
    fn metallic_smoothness_is_four_channels_with_gaps() {
        let TextureBake::Packing(packing) = Texture::MetallicSmoothness.bake() else {
            panic!("expected a packing");
        };
        assert_eq!(packing.channel_count(), 4);
        assert_eq!(
            packing.sources(),
            vec![
                attribute(METALLIC_FACTOR, false),
                ChannelSource::Zero,
                ChannelSource::Zero,
                attribute(ROUGHNESS_FACTOR, true),
            ]
        );
    }

    #[test]
    fn computed_occlusion_is_a_single_geometry_channel() {
        let TextureBake::Packing(packing) = Texture::ComputedOcclusion.bake() else {
            panic!("expected a packing");
        };
        assert_eq!(packing.sources(), vec![ChannelSource::ComputedOcclusion]);
    }

    #[test]
    fn emissive_is_the_tinted_emissive_color() {
        // The preset lowers to the emissive-color bake (emissiveFactor x
        // emissiveStrength), so a surface glows in its own emissive color.
        assert_eq!(Texture::Emissive.bake(), TextureBake::EmissiveColor);
    }
}
