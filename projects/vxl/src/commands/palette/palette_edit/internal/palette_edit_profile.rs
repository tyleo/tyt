use crate::{Profile, ProfileDescription};
use serde::Deserialize;
use std::collections::BTreeMap;

/// A `palette edit` profile, each element mirroring a flag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PaletteEditProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--values-from` per entry. The properties never travel.
    pub(crate) values_from: Vec<String>,

    /// Mirrors `--value` per entry.
    pub(crate) values: Vec<String>,

    /// Mirrors `--write-property`, an expression per property.
    pub(crate) properties: BTreeMap<String, String>,
}

impl Profile for PaletteEditProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PaletteEditProfile;

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: PaletteEditProfile = serde_json::from_str(
            r#"{
                "description": "Rusty materials roughen",
                "valuesFrom": ["tags"],
                "values": ["rust = tag == \"rust\""],
                "properties": { "roughness": "mix(roughness, 0.9, rust)" }
            }"#,
        )
        .unwrap();

        assert_eq!(
            profile.description.unwrap().as_str(),
            "Rusty materials roughen"
        );
        assert_eq!(profile.values_from, ["tags"]);
        assert_eq!(profile.values, ["rust = tag == \"rust\""]);
        assert_eq!(profile.properties["roughness"], "mix(roughness, 0.9, rust)");
    }

    #[test]
    fn an_unknown_key_errors() {
        for json in [
            r#"{ "value": [] }"#,
            r#"{ "index": "0" }"#,
            r#"{ "properties": ["roughness"] }"#,
        ] {
            assert!(
                serde_json::from_str::<PaletteEditProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
