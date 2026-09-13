use crate::material::{
    ALPHA_CUTOFF, BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, NORMAL_SCALE,
    OCCLUSION_STRENGTH, ROUGHNESS, TRANSMISSION,
};

/// The kind of a modeled material property. A custom property outside the
/// vocabulary has no kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialPropertyKind {
    /// A four-component color with straight alpha: `baseColor`.
    ColorRgba,

    /// A three-component color with no alpha: `emissiveColor`.
    ColorRgb,

    /// A scalar: every other modeled property.
    Scalar,
}

impl MaterialPropertyKind {
    /// Classifies `key` against the vocabulary, or `None` for a custom key
    /// outside it.
    pub fn of(key: &str) -> Option<Self> {
        match key {
            BASE_COLOR => Some(Self::ColorRgba),
            EMISSIVE_COLOR => Some(Self::ColorRgb),
            METALLIC | ROUGHNESS | OCCLUSION_STRENGTH | NORMAL_SCALE | EMISSIVE_STRENGTH | IOR
            | TRANSMISSION | ALPHA_CUTOFF => Some(Self::Scalar),
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
