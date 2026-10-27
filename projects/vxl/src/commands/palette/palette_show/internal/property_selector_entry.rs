use crate::commands::parse_property_selector;
use serde::Deserialize;
use voxsmith::operations::palette::PropertySelector;

/// A profile's `properties` entry, which holds the `--property` fields by name.
/// `palette` defaults to `*`, and `presentation` and `reading` default to
/// `auto`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "PropertySelectorRepr")]
pub struct PropertySelectorEntry(pub(crate) PropertySelector);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertySelectorRepr {
    palette: Option<String>,

    property: String,

    presentation: Option<String>,

    reading: Option<String>,
}

impl TryFrom<PropertySelectorRepr> for PropertySelectorEntry {
    type Error = String;

    fn try_from(repr: PropertySelectorRepr) -> Result<Self, String> {
        parse_property_selector(
            repr.palette.as_deref().unwrap_or("*"),
            &repr.property,
            repr.presentation.as_deref().unwrap_or("auto"),
            repr.reading.as_deref().unwrap_or("auto"),
        )
        .map(PropertySelectorEntry)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PropertySelectorEntry;
    use branded_id::U32Id;
    use voxsmith::{
        operations::palette::{PaletteShowPresentation, PaletteShowReading, PropertyRef},
        utilities::{IdSelector, VectorComponent},
    };

    /// The entry `json` reads into, or its error.
    fn read(json: &str) -> Result<PropertySelectorEntry, String> {
        serde_json::from_str(json).map_err(|error| error.to_string())
    }

    #[test]
    fn a_bare_property_defaults_the_other_fields() {
        let PropertySelectorEntry(selector) = read(r#"{ "property": "baseColor" }"#).unwrap();

        assert_eq!(selector.palette, IdSelector::all());
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "baseColor".to_owned(),
                component: None,
            }
        );
        assert_eq!(selector.presentation, PaletteShowPresentation::Auto);
        assert_eq!(selector.reading, PaletteShowReading::Auto);
    }

    #[test]
    fn every_field_reads_by_its_command_line_spelling() {
        let PropertySelectorEntry(selector) = read(
            r#"{ "palette": "0", "property": "baseColor.a", "presentation": "value", "reading": "srgb-hex" }"#,
        )
        .unwrap();

        assert_eq!(selector.palette, IdSelector::id(U32Id::from_u32(0)));
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "baseColor".to_owned(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(selector.presentation, PaletteShowPresentation::Value);
        assert_eq!(selector.reading, PaletteShowReading::SrgbHex);
    }

    #[test]
    fn a_missing_property_errors() {
        let error = read(r#"{ "palette": "0" }"#).unwrap_err();
        assert!(error.contains("missing field `property`"), "{error}");
    }

    #[test]
    fn an_unknown_key_or_field_value_errors() {
        let error = read(r#"{ "property": "baseColor", "format": "value" }"#).unwrap_err();
        assert!(error.contains("unknown field `format`"), "{error}");

        assert!(read(r#"{ "property": "baseColor", "reading": "rainbow" }"#).is_err());
        assert!(read(r#"{ "palette": "a", "property": "baseColor" }"#).is_err());
        assert!(read(r#"{ "palette": 0, "property": "baseColor" }"#).is_err());
    }
}
