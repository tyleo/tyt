use crate::{ChannelPacking, ChannelSource, TextureBake};
use clap::ValueEnum;

/// A named `--texture` preset, a common material-map packing. Each lowers to a
/// [`TextureBake`] with [`Texture::bake`]: `albedo` to the RGBA base color, the
/// rest to a scalar [`ChannelPacking`]. `computed-occlusion` reads occlusion
/// from the voxel geometry, so it always bakes into an unwrap layout.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Texture {
    /// RGBA base color from `rgba`. Four channels.
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
    /// Grayscale `emissive`. One channel.
    #[value(name = "emissive")]
    Emissive,
    /// Grayscale `occlusion`. One channel.
    #[value(name = "occlusion")]
    Occlusion,
    /// Grayscale occlusion computed from the voxel geometry. One channel; always
    /// an unwrap layout.
    #[value(name = "computed-occlusion")]
    ComputedOcclusion,
    /// Grayscale `roughness`. One channel.
    #[value(name = "roughness")]
    Roughness,
    /// Grayscale `smoothness`, the derived `1-roughness`. One channel.
    #[value(name = "smoothness")]
    Smoothness,
}

impl Texture {
    /// Lowers this preset to the map the bake writes.
    pub fn bake(self) -> TextureBake {
        match self {
            Texture::Albedo => TextureBake::RgbaColor,
            Texture::Orm => packing(
                attribute("occlusion", false),
                attribute("roughness", false),
                attribute("metallic", false),
                None,
            ),
            Texture::MetallicRoughness => packing(
                Some(ChannelSource::Zero),
                attribute("roughness", false),
                attribute("metallic", false),
                None,
            ),
            Texture::MetallicSmoothness => packing(
                attribute("metallic", false),
                Some(ChannelSource::Zero),
                Some(ChannelSource::Zero),
                attribute("roughness", true),
            ),
            Texture::Mse => packing(
                attribute("metallic", false),
                attribute("roughness", true),
                attribute("emissive", false),
                None,
            ),
            Texture::Emissive => packing(attribute("emissive", false), None, None, None),
            Texture::Occlusion => packing(attribute("occlusion", false), None, None, None),
            Texture::ComputedOcclusion => {
                packing(Some(ChannelSource::ComputedOcclusion), None, None, None)
            }
            Texture::Roughness => packing(attribute("roughness", false), None, None, None),
            Texture::Smoothness => packing(attribute("roughness", true), None, None, None),
        }
    }
}

/// An attribute channel source by voxj key, inverted when `invert` is set.
fn attribute(key: &str, invert: bool) -> Option<ChannelSource> {
    Some(ChannelSource::Attribute {
        key: key.to_string(),
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

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            invert,
        }
    }

    #[test]
    fn albedo_is_the_rgba_color() {
        assert_eq!(Texture::Albedo.bake(), TextureBake::RgbaColor);
    }

    #[test]
    fn mse_matches_its_texture_map_equivalent() {
        let manual = "R=metallic,G=smoothness,B=emissive"
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
                attribute("metallic", false),
                ChannelSource::Zero,
                ChannelSource::Zero,
                attribute("roughness", true),
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
}
