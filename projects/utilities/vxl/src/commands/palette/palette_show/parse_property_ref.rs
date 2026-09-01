use crate::CliValue;
use voxsmith::{PropertyRef, VectorComponent};

/// Parses the property field of a `--property` selector: `*` for every
/// property, else a key with an optional trailing `.r`/`.g`/`.b`/`.a` or
/// `.x`/`.y`/`.z`/`.w` component. A longer dotted suffix stays part of the
/// key.
pub fn parse_property_ref(text: &str) -> Result<PropertyRef, String> {
    if text == "*" {
        return Ok(PropertyRef::All);
    }

    let (key, component) = match text.rsplit_once('.') {
        Some((head, tail)) if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) => {
            (head, Some(VectorComponent::parse(tail)?))
        }
        _ => (text, None),
    };

    if key.is_empty() {
        return Err(format!("`{text}` names no property"));
    }

    Ok(PropertyRef::Key {
        key: key.to_string(),
        component,
    })
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_property_ref;
    use voxsmith::{PropertyRef, VectorComponent};

    #[test]
    fn parses_a_star() {
        assert_eq!(parse_property_ref("*").unwrap(), PropertyRef::All);
    }

    #[test]
    fn parses_a_bare_key() {
        assert_eq!(
            parse_property_ref("rgba").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn parses_a_trailing_component_from_either_alias_set() {
        assert_eq!(
            parse_property_ref("rgba.a").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(
            parse_property_ref("normal.y").unwrap(),
            PropertyRef::Key {
                key: "normal".to_string(),
                component: Some(VectorComponent::Y),
            }
        );
    }

    #[test]
    fn keeps_a_dotted_key_without_a_component() {
        assert_eq!(
            parse_property_ref("my.attr").unwrap(),
            PropertyRef::Key {
                key: "my.attr".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(parse_property_ref("rgba.q").is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(parse_property_ref("").is_err());
        assert!(parse_property_ref(".a").is_err());
    }
}
