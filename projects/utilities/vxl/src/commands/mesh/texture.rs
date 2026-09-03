use crate::commands::ChannelPacking;
use clap::ValueEnum;
use voxcore::material::{EMISSIVE_STRENGTH, METALLIC, OCCLUSION_STRENGTH, ROUGHNESS};
use voxsmith::{MaterialBake, MaterialChannel, MaterialMap, MaterialSlot};

/// A single-map material preset. The left side of `--texture-name` and each map
/// a `--texture` bakes; the bundle-inclusive `--texture` value is
/// [`TextureArg`](crate::commands::TextureArg).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
pub enum Texture {
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
}

impl Texture {
    /// The glTF material slot this preset fills, [`MaterialSlot::None`] for a
    /// preset with no standard slot.
    pub fn slot(self) -> MaterialSlot {
        match self {
            Texture::Albedo => MaterialSlot::BaseColor,
            Texture::Orm => MaterialSlot::OcclusionMetallicRoughness,
            Texture::MetallicRoughness => MaterialSlot::MetallicRoughness,
            Texture::Occlusion => MaterialSlot::Occlusion,
            Texture::Emissive => MaterialSlot::Emissive,
            Texture::MetallicSmoothness
            | Texture::Mse
            | Texture::ComputedOcclusion
            | Texture::Roughness
            | Texture::Smoothness => MaterialSlot::None,
        }
    }

    /// This preset's CLI name, as its default file-name component, e.g.
    /// `albedo`.
    pub fn cli_name(self) -> String {
        self.to_possible_value()
            .expect("every texture preset has a value")
            .get_name()
            .to_owned()
    }

    /// The map this preset bakes under the file `name`.
    pub fn map(self, name: String) -> MaterialMap {
        MaterialMap {
            name,
            slot: self.slot(),
            bake: self.bake(),
        }
    }

    /// What this preset's map writes into its image.
    pub fn bake(self) -> MaterialBake {
        match self {
            Texture::Albedo => MaterialBake::RgbaColor,

            Texture::Orm => packing(
                property(OCCLUSION_STRENGTH, false),
                property(ROUGHNESS, false),
                property(METALLIC, false),
                None,
            ),

            Texture::MetallicRoughness => packing(
                Some(MaterialChannel::Zero),
                property(ROUGHNESS, false),
                property(METALLIC, false),
                None,
            ),

            Texture::MetallicSmoothness => packing(
                property(METALLIC, false),
                Some(MaterialChannel::Zero),
                Some(MaterialChannel::Zero),
                property(ROUGHNESS, true),
            ),

            Texture::Mse => packing(
                property(METALLIC, false),
                property(ROUGHNESS, true),
                property(EMISSIVE_STRENGTH, false),
                None,
            ),

            Texture::Emissive => MaterialBake::EmissiveColor,

            Texture::Occlusion => packing(property(OCCLUSION_STRENGTH, false), None, None, None),

            Texture::ComputedOcclusion => {
                packing(Some(MaterialChannel::ComputedOcclusion), None, None, None)
            }

            Texture::Roughness => packing(property(ROUGHNESS, false), None, None, None),

            Texture::Smoothness => packing(property(ROUGHNESS, true), None, None, None),
        }
    }
}

/// A property channel by voxj key, inverted when `invert` is set.
fn property(key: &str, invert: bool) -> Option<MaterialChannel> {
    Some(MaterialChannel::Property {
        key: key.to_string(),
        component: None,
        invert,
    })
}

/// A packing bake from per-channel sources.
fn packing(
    r: Option<MaterialChannel>,
    g: Option<MaterialChannel>,
    b: Option<MaterialChannel>,
    a: Option<MaterialChannel>,
) -> MaterialBake {
    MaterialBake::Packing(ChannelPacking::new(r, g, b, a).sources())
}

#[cfg(test)]
mod tests {
    use crate::commands::{ChannelPacking, Texture};
    use voxsmith::{
        MaterialBake, MaterialChannel,
        voxcore::material::{METALLIC, ROUGHNESS},
    };

    fn property(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Property {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    #[test]
    fn albedo_is_the_rgba_color() {
        assert_eq!(Texture::Albedo.bake(), MaterialBake::RgbaColor);
    }

    #[test]
    fn mse_matches_its_texture_map_equivalent() {
        let manual = "R=metallic,G=1-roughness,B=emissiveStrength"
            .parse::<ChannelPacking>()
            .unwrap();
        assert_eq!(Texture::Mse.bake(), MaterialBake::Packing(manual.sources()));
    }

    #[test]
    fn metallic_smoothness_is_four_channels_with_gaps() {
        let MaterialBake::Packing(channels) = Texture::MetallicSmoothness.bake() else {
            panic!("expected a packing");
        };
        assert_eq!(
            channels,
            vec![
                property(METALLIC, false),
                MaterialChannel::Zero,
                MaterialChannel::Zero,
                property(ROUGHNESS, true),
            ]
        );
    }

    #[test]
    fn computed_occlusion_is_a_single_geometry_channel() {
        let MaterialBake::Packing(channels) = Texture::ComputedOcclusion.bake() else {
            panic!("expected a packing");
        };
        assert_eq!(channels, vec![MaterialChannel::ComputedOcclusion]);
    }

    #[test]
    fn emissive_is_the_tinted_emissive_color() {
        // The preset lowers to the emissive-color bake (emissiveColor x
        // emissiveStrength), so a surface glows in its own emissive color.
        assert_eq!(Texture::Emissive.bake(), MaterialBake::EmissiveColor);
    }
}
