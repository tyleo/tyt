use crate::material::{
    BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH, ROUGHNESS,
    TRANSMISSION,
};

/// The kind of a recommended material property: a four- or three-component
/// color, or a scalar. A custom property outside the vocabulary has no kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialPropertyKind {
    /// A four-component color with straight alpha: `baseColor`.
    ColorRgba,

    /// A three-component color with no alpha: `emissiveColor`.
    ColorRgb,

    /// A scalar: `metallic`, `roughness`, `occlusionStrength`,
    /// `emissiveStrength`, `ior`, or `transmission`.
    Scalar,
}

impl MaterialPropertyKind {
    /// Classifies `key` against the recommended vocabulary, or `None` for a
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
    use crate::material::{BASE_COLOR, EMISSIVE_COLOR, MaterialPropertyKind, OCCLUSION_STRENGTH};

    #[test]
    fn classifies_the_two_colors_and_a_scalar() {
        assert_eq!(
            MaterialPropertyKind::of(BASE_COLOR),
            Some(MaterialPropertyKind::ColorRgba)
        );
        assert_eq!(
            MaterialPropertyKind::of(EMISSIVE_COLOR),
            Some(MaterialPropertyKind::ColorRgb)
        );
        assert_eq!(
            MaterialPropertyKind::of(OCCLUSION_STRENGTH),
            Some(MaterialPropertyKind::Scalar)
        );
    }

    #[test]
    fn a_custom_key_is_outside_the_vocabulary() {
        assert_eq!(MaterialPropertyKind::of("subsurface"), None);
    }
}
