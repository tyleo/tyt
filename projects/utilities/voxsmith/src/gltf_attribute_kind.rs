use crate::{
    BASE_COLOR_FACTOR, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, IOR, METALLIC_FACTOR,
    OCCLUSION_STRENGTH, ROUGHNESS_FACTOR, TRANSMISSION_FACTOR,
};

/// The kind of a recommended glTF attribute: a four- or three-component color,
/// or a scalar. The vocabulary the voxj unbound-default rule keys on: an absent
/// glTF attribute takes its spec default, a custom one has none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GltfAttributeKind {
    /// A four-component color with straight alpha: `baseColorFactor`.
    ColorRgba,

    /// A three-component color with no alpha: `emissiveFactor`.
    ColorRgb,

    /// A scalar: `metallicFactor`, `roughnessFactor`, `occlusionStrength`,
    /// `emissiveStrength`, `ior`, or `transmissionFactor`.
    Scalar,
}

impl GltfAttributeKind {
    /// Classifies `key` against the recommended glTF vocabulary, or `None` for a
    /// custom key outside it.
    pub fn of(key: &str) -> Option<Self> {
        match key {
            BASE_COLOR_FACTOR => Some(Self::ColorRgba),
            EMISSIVE_FACTOR => Some(Self::ColorRgb),
            METALLIC_FACTOR | ROUGHNESS_FACTOR | OCCLUSION_STRENGTH | EMISSIVE_STRENGTH | IOR
            | TRANSMISSION_FACTOR => Some(Self::Scalar),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, EMISSIVE_FACTOR, GltfAttributeKind, OCCLUSION_STRENGTH};

    #[test]
    fn classifies_the_two_colors_and_a_scalar() {
        assert_eq!(
            GltfAttributeKind::of(BASE_COLOR_FACTOR),
            Some(GltfAttributeKind::ColorRgba)
        );
        assert_eq!(
            GltfAttributeKind::of(EMISSIVE_FACTOR),
            Some(GltfAttributeKind::ColorRgb)
        );
        assert_eq!(
            GltfAttributeKind::of(OCCLUSION_STRENGTH),
            Some(GltfAttributeKind::Scalar)
        );
    }

    #[test]
    fn a_custom_key_is_outside_the_vocabulary() {
        assert_eq!(GltfAttributeKind::of("subsurface"), None);
    }
}
