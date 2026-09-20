use crate::operations::mesh::Transfer;

/// A modeled material field a slot write fills, under the property names of
/// the document's material model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SlotProperty {
    AlphaCutoff,
    AlphaMode,
    BaseColorFactor,
    BaseColorTexture,
    DoubleSided,
    EmissiveFactor,
    EmissiveStrength,
    EmissiveTexture,
    Ior,
    MetallicFactor,
    MetallicRoughnessTexture,
    NormalScale,
    NormalTexture,
    OcclusionStrength,
    OcclusionTexture,
    RoughnessFactor,
    TransmissionFactor,
    TransmissionTexture,
}

impl SlotProperty {
    const ALL: [SlotProperty; 18] = [
        SlotProperty::AlphaCutoff,
        SlotProperty::AlphaMode,
        SlotProperty::BaseColorFactor,
        SlotProperty::BaseColorTexture,
        SlotProperty::DoubleSided,
        SlotProperty::EmissiveFactor,
        SlotProperty::EmissiveStrength,
        SlotProperty::EmissiveTexture,
        SlotProperty::Ior,
        SlotProperty::MetallicFactor,
        SlotProperty::MetallicRoughnessTexture,
        SlotProperty::NormalScale,
        SlotProperty::NormalTexture,
        SlotProperty::OcclusionStrength,
        SlotProperty::OcclusionTexture,
        SlotProperty::RoughnessFactor,
        SlotProperty::TransmissionFactor,
        SlotProperty::TransmissionTexture,
    ];

    /// The property named `name`, or `None` outside the vocabulary.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        SlotProperty::ALL
            .into_iter()
            .find(|property| property.name() == name)
    }

    /// The property's name.
    pub(crate) fn name(self) -> &'static str {
        match self {
            SlotProperty::AlphaCutoff => "alphaCutoff",
            SlotProperty::AlphaMode => "alphaMode",
            SlotProperty::BaseColorFactor => "baseColorFactor",
            SlotProperty::BaseColorTexture => "baseColorTexture",
            SlotProperty::DoubleSided => "doubleSided",
            SlotProperty::EmissiveFactor => "emissiveFactor",
            SlotProperty::EmissiveStrength => "emissiveStrength",
            SlotProperty::EmissiveTexture => "emissiveTexture",
            SlotProperty::Ior => "ior",
            SlotProperty::MetallicFactor => "metallicFactor",
            SlotProperty::MetallicRoughnessTexture => "metallicRoughnessTexture",
            SlotProperty::NormalScale => "normalScale",
            SlotProperty::NormalTexture => "normalTexture",
            SlotProperty::OcclusionStrength => "occlusionStrength",
            SlotProperty::OcclusionTexture => "occlusionTexture",
            SlotProperty::RoughnessFactor => "roughnessFactor",
            SlotProperty::TransmissionFactor => "transmissionFactor",
            SlotProperty::TransmissionTexture => "transmissionTexture",
        }
    }

    /// Whether the property takes an image.
    pub(crate) fn is_texture(self) -> bool {
        matches!(
            self,
            SlotProperty::BaseColorTexture
                | SlotProperty::EmissiveTexture
                | SlotProperty::MetallicRoughnessTexture
                | SlotProperty::NormalTexture
                | SlotProperty::OcclusionTexture
                | SlotProperty::TransmissionTexture
        )
    }

    /// The encoding a texture property fixes for its image.
    pub(crate) fn transfer(self) -> Transfer {
        match self {
            SlotProperty::BaseColorTexture | SlotProperty::EmissiveTexture => Transfer::Srgb,

            SlotProperty::MetallicRoughnessTexture
            | SlotProperty::NormalTexture
            | SlotProperty::OcclusionTexture
            | SlotProperty::TransmissionTexture => Transfer::Linear,

            _ => unreachable!("a factor fixes no image encoding"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{SlotProperty, Transfer};

    #[test]
    fn every_property_parses_from_its_name_and_the_textures_take_images() {
        for property in SlotProperty::ALL {
            assert_eq!(SlotProperty::parse(property.name()), Some(property));
            assert_eq!(
                property.is_texture(),
                property.name().ends_with("Texture"),
                "{property:?}"
            );
        }

        assert_eq!(SlotProperty::parse("subsurface"), None);
    }

    #[test]
    fn the_color_textures_are_srgb_and_the_data_textures_linear() {
        assert_eq!(SlotProperty::BaseColorTexture.transfer(), Transfer::Srgb);
        assert_eq!(SlotProperty::EmissiveTexture.transfer(), Transfer::Srgb);
        assert_eq!(
            SlotProperty::MetallicRoughnessTexture.transfer(),
            Transfer::Linear
        );
        assert_eq!(SlotProperty::OcclusionTexture.transfer(), Transfer::Linear);
    }
}
