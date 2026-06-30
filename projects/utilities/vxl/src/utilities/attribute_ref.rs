use crate::ColorComponent;

/// The attribute field of an `--attribute` selector: one attribute key,
/// optionally narrowed to a color component, or every attribute of the palette.
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeRef {
    /// Every attribute of the palette.
    All,
    /// One attribute by key, optionally narrowed to one color `component`.
    Key {
        /// The attribute key, with any trailing `.component` stripped off.
        key: String,
        /// The color component to read, or `None` for the whole value.
        component: Option<ColorComponent>,
    },
}

impl AttributeRef {
    /// Parses the attribute field: `*` for every attribute, else a key with an
    /// optional trailing `.r`/`.g`/`.b`/`.a` color component. A longer dotted
    /// suffix stays part of the key.
    pub fn parse(value: &str) -> Result<Self, String> {
        if value == "*" {
            return Ok(AttributeRef::All);
        }
        let (key, component) = match value.rsplit_once('.') {
            Some((head, tail))
                if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                (head, Some(tail.parse::<ColorComponent>()?))
            }
            _ => (value, None),
        };
        if key.is_empty() {
            return Err(format!("`{value}` names no attribute"));
        }
        Ok(AttributeRef::Key {
            key: key.to_string(),
            component,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributeRef, ColorComponent};

    #[test]
    fn parses_a_star() {
        assert_eq!(AttributeRef::parse("*").unwrap(), AttributeRef::All);
    }

    #[test]
    fn parses_a_bare_key() {
        assert_eq!(
            AttributeRef::parse("rgba").unwrap(),
            AttributeRef::Key {
                key: "rgba".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn parses_a_trailing_color_component() {
        assert_eq!(
            AttributeRef::parse("rgba.a").unwrap(),
            AttributeRef::Key {
                key: "rgba".to_string(),
                component: Some(ColorComponent::A),
            }
        );
    }

    #[test]
    fn keeps_a_dotted_key_without_a_component() {
        assert_eq!(
            AttributeRef::parse("my.attr").unwrap(),
            AttributeRef::Key {
                key: "my.attr".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(AttributeRef::parse("rgba.z").is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(AttributeRef::parse("").is_err());
        assert!(AttributeRef::parse(".a").is_err());
    }
}
