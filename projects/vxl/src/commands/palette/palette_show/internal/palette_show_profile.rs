use crate::{
    Profile, ProfileDescription,
    commands::{PaletteShowLayoutEntry, PropertySelectorEntry},
};
use serde::Deserialize;

/// A `palette show` profile, each element mirroring a flag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PaletteShowProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--properties-from` per entry. The layout never travels.
    pub(crate) properties_from: Vec<String>,

    /// Mirrors `--property` per entry.
    pub(crate) properties: Vec<PropertySelectorEntry>,

    /// Mirrors `--layout` with the display flags that layout takes.
    pub(crate) layout: Option<PaletteShowLayoutEntry>,
}

impl Profile for PaletteShowProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{PaletteShowLayoutEntry, PaletteShowProfile};
    use voxsmith::operations::palette::{PaletteShowLayout, PaletteShowTableShape};

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: PaletteShowProfile = serde_json::from_str(
            r#"{
                "propertiesFrom": ["pbr"],
                "properties": [{ "property": "baseColor" }, { "property": "metallic" }],
                "layout": { "kind": "md-tables", "tableShape": "flat" }
            }"#,
        )
        .unwrap();

        assert_eq!(profile.properties_from, ["pbr"]);
        assert_eq!(profile.properties.len(), 2);
        assert_eq!(
            profile.layout,
            Some(PaletteShowLayoutEntry {
                table_shape: Some(PaletteShowTableShape::Flat),
                ..PaletteShowLayoutEntry::from(PaletteShowLayout::MdTables)
            })
        );
    }

    #[test]
    fn an_unknown_key_errors() {
        for json in [
            r#"{ "property": [] }"#,
            r#"{ "tableShape": "flat" }"#,
            r#"{ "width": 80 }"#,
        ] {
            assert!(
                serde_json::from_str::<PaletteShowProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
