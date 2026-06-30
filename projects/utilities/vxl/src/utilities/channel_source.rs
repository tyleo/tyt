use std::str::FromStr;

/// One channel's value in a material map: the right-hand side of an `R=`, `G=`,
/// `B=`, or `A=` entry in `--texture-map`, and the building block the
/// `--texture` presets are defined in terms of. A channel reads an attribute by
/// its voxj key, optionally inverted as `1-<attribute>`, the constant `0` or
/// `1`, or `computed-occlusion` derived from the voxel geometry.
///
/// `smoothness` is the derived `1-roughness`, so `smoothness` and `1-roughness`
/// are interchangeable, as are `1-smoothness` and `roughness`; all canonicalize
/// to the `roughness` attribute with the matching inversion. The attribute key
/// is held as a string because the voxj format stores attributes generically,
/// so a packing may read any attribute, not only the recommended ones.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelSource {
    /// An attribute value by its voxj key, inverted to `1 - value` when `invert`
    /// is set.
    Attribute { key: String, invert: bool },
    /// The constant `0`.
    Zero,
    /// The constant `1`.
    One,
    /// Occlusion computed from the voxel geometry. Always bakes into an unwrap
    /// layout.
    ComputedOcclusion,
}

impl FromStr for ChannelSource {
    type Err = String;

    /// Parses one channel expression. `0` and `1` are the constants,
    /// `computed-occlusion` the geometry-derived occlusion, a leading `1-`
    /// inverts an attribute, and `smoothness` canonicalizes to inverted
    /// `roughness`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "0" => Ok(ChannelSource::Zero),
            "1" => Ok(ChannelSource::One),
            "computed-occlusion" => Ok(ChannelSource::ComputedOcclusion),
            _ => {
                let (name, invert) = match value.strip_prefix("1-") {
                    Some(rest) => (rest, true),
                    None => (value, false),
                };
                if name.is_empty() {
                    return Err(format!("`{value}` names no attribute"));
                }
                // `smoothness` is the derived `1-roughness`, so reading it flips
                // the inversion against the stored `roughness` attribute.
                let (key, invert) = if name == "smoothness" {
                    ("roughness".to_string(), !invert)
                } else {
                    (name.to_string(), invert)
                };
                Ok(ChannelSource::Attribute { key, invert })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ChannelSource;

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            invert,
        }
    }

    #[test]
    fn parses_constants_and_computed_occlusion() {
        assert_eq!("0".parse::<ChannelSource>().unwrap(), ChannelSource::Zero);
        assert_eq!("1".parse::<ChannelSource>().unwrap(), ChannelSource::One);
        assert_eq!(
            "computed-occlusion".parse::<ChannelSource>().unwrap(),
            ChannelSource::ComputedOcclusion
        );
    }

    #[test]
    fn parses_attribute_and_inverse() {
        assert_eq!(
            "metallic".parse::<ChannelSource>().unwrap(),
            attribute("metallic", false)
        );
        assert_eq!(
            "1-metallic".parse::<ChannelSource>().unwrap(),
            attribute("metallic", true)
        );
    }

    #[test]
    fn smoothness_canonicalizes_to_inverted_roughness() {
        assert_eq!(
            "roughness".parse::<ChannelSource>().unwrap(),
            attribute("roughness", false)
        );
        assert_eq!(
            "1-roughness".parse::<ChannelSource>().unwrap(),
            attribute("roughness", true)
        );
        assert_eq!(
            "smoothness".parse::<ChannelSource>().unwrap(),
            attribute("roughness", true)
        );
        assert_eq!(
            "1-smoothness".parse::<ChannelSource>().unwrap(),
            attribute("roughness", false)
        );
    }

    #[test]
    fn rejects_empty_attribute() {
        assert!("".parse::<ChannelSource>().is_err());
        assert!("1-".parse::<ChannelSource>().is_err());
    }
}
