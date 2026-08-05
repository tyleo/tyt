use crate::VectorComponent;

/// The property field of a `--property` selector: one property key,
/// optionally narrowed to one vector component, or every property of the
/// palette.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyRef {
    /// Every property of the palette.
    All,
    /// One property by key, optionally narrowed to one vector `component`.
    Key {
        /// The property key, with any trailing `.component` stripped off.
        key: String,
        /// The vector component to read, or `None` for the whole value.
        component: Option<VectorComponent>,
    },
}

impl PropertyRef {
    /// Parses the property field: `*` for every property, else a key with an
    /// optional trailing `.r`/`.g`/`.b`/`.a` or `.x`/`.y`/`.z`/`.w`
    /// component. A longer dotted suffix stays part of the key.
    pub fn parse(value: &str) -> Result<Self, String> {
        if value == "*" {
            return Ok(PropertyRef::All);
        }
        let (key, component) = match value.rsplit_once('.') {
            Some((head, tail))
                if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                (head, Some(tail.parse::<VectorComponent>()?))
            }
            _ => (value, None),
        };
        if key.is_empty() {
            return Err(format!("`{value}` names no property"));
        }
        Ok(PropertyRef::Key {
            key: key.to_string(),
            component,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{VectorComponent, commands::PropertyRef};

    #[test]
    fn parses_a_star() {
        assert_eq!(PropertyRef::parse("*").unwrap(), PropertyRef::All);
    }

    #[test]
    fn parses_a_bare_key() {
        assert_eq!(
            PropertyRef::parse("rgba").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn parses_a_trailing_component_from_either_alias_set() {
        assert_eq!(
            PropertyRef::parse("rgba.a").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(
            PropertyRef::parse("normal.y").unwrap(),
            PropertyRef::Key {
                key: "normal".to_string(),
                component: Some(VectorComponent::Y),
            }
        );
    }

    #[test]
    fn keeps_a_dotted_key_without_a_component() {
        assert_eq!(
            PropertyRef::parse("my.attr").unwrap(),
            PropertyRef::Key {
                key: "my.attr".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(PropertyRef::parse("rgba.q").is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(PropertyRef::parse("").is_err());
        assert!(PropertyRef::parse(".a").is_err());
    }
}
