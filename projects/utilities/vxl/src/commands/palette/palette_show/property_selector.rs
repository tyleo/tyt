use crate::commands::{PaletteRef, PaletteShowFormat, PropertyRef};
use clap::ValueEnum;

/// A `--property <palette> <property> <format>` selector for `palette show`,
/// naming a value collection. `*` matches every palette or property.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySelector {
    /// Which palette, one index or every palette.
    pub(crate) palette: PaletteRef,
    /// Which property, one key with an optional color component or every
    /// property.
    pub(crate) property: PropertyRef,
    /// How each value in the collection renders.
    pub(crate) format: PaletteShowFormat,
}

impl PropertySelector {
    /// Parses one selector from its three fields.
    pub fn parse(palette: &str, property: &str, format: &str) -> Result<Self, String> {
        Ok(PropertySelector {
            palette: PaletteRef::parse(palette)?,
            property: PropertyRef::parse(property)?,
            format: PaletteShowFormat::from_str(format, false)?,
        })
    }

    /// The default selector when no `--property` is given: every palette and
    /// every property in the `auto` format, the same as `'*' '*' auto`.
    pub fn default_all_auto() -> Self {
        PropertySelector {
            palette: PaletteRef::All,
            property: PropertyRef::All,
            format: PaletteShowFormat::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ColorComponent,
        commands::{PaletteRef, PaletteShowFormat, PropertyRef, PropertySelector},
    };

    #[test]
    fn parses_a_full_selector() {
        let selector = PropertySelector::parse("0", "rgba.a", "value").unwrap();
        assert_eq!(selector.palette, PaletteRef::Index(0));
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(ColorComponent::A),
            }
        );
        assert!(matches!(selector.format, PaletteShowFormat::Value));
    }

    #[test]
    fn parses_stars() {
        let selector = PropertySelector::parse("*", "*", "swatch").unwrap();
        assert_eq!(selector.palette, PaletteRef::All);
        assert_eq!(selector.property, PropertyRef::All);
    }

    #[test]
    fn rejects_an_unknown_format() {
        assert!(PropertySelector::parse("0", "rgba", "rainbow").is_err());
    }
}
