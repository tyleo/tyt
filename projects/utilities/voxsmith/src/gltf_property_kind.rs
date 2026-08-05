use crate::{
    BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH, ROUGHNESS,
    TRANSMISSION,
};

/// The kind of a recommended glTF property: a four- or three-component color,
/// or a scalar. The vocabulary the voxj unbound-default rule keys on: an absent
/// glTF property takes its spec default, a custom one has none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GltfPropertyKind {
    /// A four-component color with straight alpha: `baseColor`.
    ColorRgba,

    /// A three-component color with no alpha: `emissiveColor`.
    ColorRgb,

    /// A scalar: `metallic`, `roughness`, `occlusionStrength`,
    /// `emissiveStrength`, `ior`, or `transmission`.
    Scalar,
}

impl GltfPropertyKind {
    /// Classifies `key` against the recommended glTF vocabulary, or `None` for a
    /// custom key outside it.
    pub fn of(key: &str) -> Option<Self> {
        match key {
            BASE_COLOR => Some(Self::ColorRgba),
            EMISSIVE_COLOR => Some(Self::ColorRgb),
            METALLIC | ROUGHNESS | OCCLUSION_STRENGTH | EMISSIVE_STRENGTH | IOR | TRANSMISSION => {
                Some(Self::Scalar)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR, EMISSIVE_COLOR, GltfPropertyKind, OCCLUSION_STRENGTH};

    #[test]
    fn classifies_the_two_colors_and_a_scalar() {
        assert_eq!(
            GltfPropertyKind::of(BASE_COLOR),
            Some(GltfPropertyKind::ColorRgba)
        );
        assert_eq!(
            GltfPropertyKind::of(EMISSIVE_COLOR),
            Some(GltfPropertyKind::ColorRgb)
        );
        assert_eq!(
            GltfPropertyKind::of(OCCLUSION_STRENGTH),
            Some(GltfPropertyKind::Scalar)
        );
    }

    #[test]
    fn a_custom_key_is_outside_the_vocabulary() {
        assert_eq!(GltfPropertyKind::of("subsurface"), None);
    }
}
