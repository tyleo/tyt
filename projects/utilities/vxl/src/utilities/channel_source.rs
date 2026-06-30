use crate::ColorComponent;
use std::str::FromStr;

/// One channel's value in a material map: an attribute by name, optionally one
/// color component of it, optionally inverted as `1-<attribute>`, the constant
/// `0` or `1`, or `computed-occlusion` derived from the voxel geometry.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelSource {
    /// An attribute value by name, optionally one color `component` of it,
    /// inverted to `1 - value` when `invert` is set.
    Attribute {
        key: String,
        component: Option<ColorComponent>,
        invert: bool,
    },
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
    /// inverts an attribute, a trailing `.r`/`.g`/`.b`/`.a` reads one color
    /// component, and `smoothness` canonicalizes to inverted `roughness`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "0" => return Ok(ChannelSource::Zero),
            "1" => return Ok(ChannelSource::One),
            "computed-occlusion" => return Ok(ChannelSource::ComputedOcclusion),
            _ => {}
        }
        let (body, invert) = match value.strip_prefix("1-") {
            Some(rest) => (rest, true),
            None => (value, false),
        };
        // A trailing `.r`/`.g`/`.b`/`.a` selects one color component; a longer
        // suffix is part of the name, so dotted keys pass through unsplit.
        let (name, component) = match body.rsplit_once('.') {
            Some((head, tail))
                if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                (head, Some(tail.parse::<ColorComponent>()?))
            }
            _ => (body, None),
        };
        if name.is_empty() {
            return Err(format!("`{value}` names no attribute"));
        }
        // `smoothness` is the derived `1-roughness`, so reading it flips the
        // inversion against the stored `roughness` attribute.
        let (key, invert) = if name == "smoothness" {
            ("roughness".to_string(), !invert)
        } else {
            (name.to_string(), invert)
        };
        Ok(ChannelSource::Attribute {
            key,
            component,
            invert,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChannelSource, ColorComponent};

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    fn component(key: &str, component: ColorComponent, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component: Some(component),
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
    fn parses_color_components() {
        assert_eq!(
            "rgba.r".parse::<ChannelSource>().unwrap(),
            component("rgba", ColorComponent::R, false)
        );
        assert_eq!(
            "1-rgba.a".parse::<ChannelSource>().unwrap(),
            component("rgba", ColorComponent::A, true)
        );
    }

    #[test]
    fn keeps_dotted_keys_without_a_component() {
        assert_eq!(
            "my.attr".parse::<ChannelSource>().unwrap(),
            attribute("my.attr", false)
        );
    }

    #[test]
    fn rejects_empty_attribute() {
        assert!("".parse::<ChannelSource>().is_err());
        assert!("1-".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn rejects_unknown_color_component() {
        assert!("rgba.z".parse::<ChannelSource>().is_err());
    }
}
