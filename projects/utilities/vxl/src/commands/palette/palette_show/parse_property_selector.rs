use crate::{
    CliValue,
    commands::{parse_palette_ref, parse_property_ref},
};
use voxsmith::{PaletteShowPresentation, PaletteShowReading, PropertySelector};

/// Parses one `--property <palette> <property> <presentation> <reading>`
/// selector for `palette show` from its four fields. `*` matches every palette
/// or property.
pub fn parse_property_selector(
    palette: &str,
    property: &str,
    presentation: &str,
    reading: &str,
) -> Result<PropertySelector, String> {
    Ok(PropertySelector {
        palette: parse_palette_ref(palette)?,
        property: parse_property_ref(property)?,
        presentation: PaletteShowPresentation::parse(presentation)?,
        reading: PaletteShowReading::parse(reading)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_property_selector;
    use voxsmith::{
        PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef, VectorComponent,
    };

    #[test]
    fn parses_a_full_selector() {
        let selector = parse_property_selector("0", "rgba.a", "value", "srgb-hex").unwrap();

        assert_eq!(selector.palette, PaletteRef::Index(0));
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(selector.presentation, PaletteShowPresentation::Value);
        assert_eq!(selector.reading, PaletteShowReading::SrgbHex);
    }

    #[test]
    fn parses_stars() {
        let selector = parse_property_selector("*", "*", "swatch", "auto").unwrap();

        assert_eq!(selector.palette, PaletteRef::All);
        assert_eq!(selector.property, PropertyRef::All);
    }

    #[test]
    fn rejects_an_unknown_presentation() {
        assert!(parse_property_selector("0", "rgba", "rainbow", "auto").is_err());
    }

    #[test]
    fn rejects_an_unknown_reading() {
        assert!(parse_property_selector("0", "rgba", "value", "rainbow").is_err());
    }
}
