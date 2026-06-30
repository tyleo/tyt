use crate::{AttributeRef, PaletteRef, PaletteShowFormat};
use clap::ValueEnum;

/// A `--attribute <palette> <attribute> <format>` selector for `palette show`,
/// naming a value collection. `*` matches every palette or attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeSelector {
    /// Which palette, one index or every palette.
    pub(crate) palette: PaletteRef,
    /// Which attribute, one key with an optional color component or every
    /// attribute.
    pub(crate) attribute: AttributeRef,
    /// How each value in the collection renders.
    pub(crate) format: PaletteShowFormat,
}

impl AttributeSelector {
    /// Parses one selector from its three fields.
    pub fn parse(palette: &str, attribute: &str, format: &str) -> Result<Self, String> {
        Ok(AttributeSelector {
            palette: PaletteRef::parse(palette)?,
            attribute: AttributeRef::parse(attribute)?,
            format: PaletteShowFormat::from_str(format, false)?,
        })
    }

    /// The default selector when no `--attribute` is given: every palette's
    /// `rgba` as swatches.
    pub fn default_rgba_swatch() -> Self {
        AttributeSelector {
            palette: PaletteRef::All,
            attribute: AttributeRef::Key {
                key: "rgba".to_string(),
                component: None,
            },
            format: PaletteShowFormat::Swatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributeRef, AttributeSelector, ColorComponent, PaletteRef, PaletteShowFormat};

    #[test]
    fn parses_a_full_selector() {
        let selector = AttributeSelector::parse("0", "rgba.a", "value").unwrap();
        assert_eq!(selector.palette, PaletteRef::Index(0));
        assert_eq!(
            selector.attribute,
            AttributeRef::Key {
                key: "rgba".to_string(),
                component: Some(ColorComponent::A),
            }
        );
        assert!(matches!(selector.format, PaletteShowFormat::Value));
    }

    #[test]
    fn parses_stars() {
        let selector = AttributeSelector::parse("*", "*", "swatch").unwrap();
        assert_eq!(selector.palette, PaletteRef::All);
        assert_eq!(selector.attribute, AttributeRef::All);
    }

    #[test]
    fn rejects_an_unknown_format() {
        assert!(AttributeSelector::parse("0", "rgba", "rainbow").is_err());
    }
}
