use crate::commands::{PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef};
use clap::ValueEnum;

/// A `--property <palette> <property> <presentation> <reading>` selector for
/// `palette show`, naming a value collection. `*` matches every palette or
/// property.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySelector {
    /// Which palette, one index or every palette.
    pub(crate) palette: PaletteRef,
    /// Which property, one key with an optional vector component or every
    /// property.
    pub(crate) property: PropertyRef,
    /// What renders for each value in the collection.
    pub(crate) presentation: PaletteShowPresentation,
    /// How each value's numbers spell.
    pub(crate) reading: PaletteShowReading,
}

impl PropertySelector {
    /// Parses one selector from its four fields.
    pub fn parse(
        palette: &str,
        property: &str,
        presentation: &str,
        reading: &str,
    ) -> Result<Self, String> {
        Ok(PropertySelector {
            palette: PaletteRef::parse(palette)?,
            property: PropertyRef::parse(property)?,
            presentation: PaletteShowPresentation::from_str(presentation, false)?,
            reading: PaletteShowReading::from_str(reading, false)?,
        })
    }

    /// The default selector when no `--property` is given: every palette and
    /// every property under the `auto` presentation and reading, the same as
    /// `'*' '*' auto auto`.
    pub fn default_all_auto() -> Self {
        PropertySelector {
            palette: PaletteRef::All,
            property: PropertyRef::All,
            presentation: PaletteShowPresentation::Auto,
            reading: PaletteShowReading::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        VectorComponent,
        commands::{
            PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef, PropertySelector,
        },
    };

    #[test]
    fn parses_a_full_selector() {
        let selector = PropertySelector::parse("0", "rgba.a", "value", "srgb-hex").unwrap();
        assert_eq!(selector.palette, PaletteRef::Index(0));
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert!(matches!(
            selector.presentation,
            PaletteShowPresentation::Value
        ));
        assert!(matches!(selector.reading, PaletteShowReading::SrgbHex));
    }

    #[test]
    fn parses_stars() {
        let selector = PropertySelector::parse("*", "*", "swatch", "auto").unwrap();
        assert_eq!(selector.palette, PaletteRef::All);
        assert_eq!(selector.property, PropertyRef::All);
    }

    #[test]
    fn rejects_an_unknown_presentation() {
        assert!(PropertySelector::parse("0", "rgba", "rainbow", "auto").is_err());
    }

    #[test]
    fn rejects_an_unknown_reading() {
        assert!(PropertySelector::parse("0", "rgba", "value", "rainbow").is_err());
    }
}
