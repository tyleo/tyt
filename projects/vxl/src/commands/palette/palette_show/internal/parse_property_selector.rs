use crate::{CliValue, parse_id_selector};
use voxsmith::{
    operations::palette::{
        PaletteShowPresentation, PaletteShowReading, PropertyRef, PropertySelector,
    },
    utilities::VectorComponent,
};

/// Parses one `--property <palette> <property> <presentation> <reading>`
/// selector for `palette show` from its four fields. The palette field takes
/// an id, an `a-b` range, or `*`. `*` matches every palette or property.
pub fn parse_property_selector(
    palette: &str,
    property: &str,
    presentation: &str,
    reading: &str,
) -> Result<PropertySelector, String> {
    Ok(PropertySelector {
        palette: parse_id_selector(palette)?,
        property: parse_property_ref(property)?,
        presentation: PaletteShowPresentation::parse(presentation)?,
        reading: PaletteShowReading::parse(reading)?,
    })
}

/// Parses the property field of a `--property` selector: `*` for every
/// property, else a key with an optional trailing `.r`/`.g`/`.b`/`.a` or
/// `.x`/`.y`/`.z`/`.w` component. A longer dotted suffix stays part of the
/// key.
fn parse_property_ref(text: &str) -> Result<PropertyRef, String> {
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
    use crate::commands::{
        palette::palette_show::internal::parse_property_selector::parse_property_ref,
        parse_property_selector,
    };
    use branded_id::U32Id;
    use voxsmith::{
        operations::palette::{PaletteShowPresentation, PaletteShowReading, PropertyRef},
        utilities::{IdSelector, VectorComponent},
    };

    #[test]
    fn parses_a_full_selector() {
        let selector = parse_property_selector("0", "rgba.a", "value", "srgb-hex").unwrap();

        assert_eq!(selector.palette, IdSelector::id(U32Id::from_u32(0)));
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

        assert_eq!(selector.palette, IdSelector::all());
        assert_eq!(selector.property, PropertyRef::All);
    }

    #[test]
    fn parses_a_palette_range() {
        let selector = parse_property_selector("1-3", "*", "value", "auto").unwrap();

        assert_eq!(
            selector.palette,
            IdSelector::range(U32Id::from_u32(1)..=U32Id::from_u32(3)).unwrap()
        );
    }

    #[test]
    fn rejects_an_unknown_presentation() {
        assert!(parse_property_selector("0", "rgba", "rainbow", "auto").is_err());
    }

    #[test]
    fn rejects_an_unknown_reading() {
        assert!(parse_property_selector("0", "rgba", "value", "rainbow").is_err());
    }

    #[test]
    fn rejects_a_bad_palette() {
        assert!(parse_property_selector("a", "rgba", "value", "auto").is_err());
        assert!(parse_property_selector("3-1", "rgba", "value", "auto").is_err());
    }

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
