use crate::commands::PaletteEditProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `palette.edit` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct PaletteEditConfig {
    pub(crate) profiles: BTreeMap<String, PaletteEditProfile>,
}

#[cfg(test)]
mod tests {
    use crate::commands::{PaletteConfig, PaletteEditConfig};
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `palette.edit` section `text` holds.
    fn read(text: &str) -> IOResult<PaletteEditConfig> {
        let config: Option<PaletteConfig> =
            JsoncCodec.deserialize_prefs(text.as_bytes(), "palette")?;

        Ok(config.expect("the text holds a palette section").edit)
    }

    #[test]
    fn the_section_reads_its_profiles_by_name() {
        let config = read(
            r#"{
  "palette": {
    "edit": {
      "profiles": {
        // A mixin.
        "tags": { "values": ["rust = tag == \"rust\""] },
        "weathered": {
          "valuesFrom": ["tags"],
          "properties": { "roughness": "mix(roughness, 0.9, rust)" },
        },
      },
    },
  },
}"#,
        )
        .unwrap();

        let names: Vec<_> = config.profiles.keys().collect();
        assert_eq!(names, ["tags", "weathered"]);
        assert_eq!(config.profiles["weathered"].values_from, ["tags"]);
    }

    #[test]
    fn an_empty_section_holds_no_profiles() {
        let config = read(r#"{ "palette": { "edit": {} } }"#).unwrap();
        assert_eq!(config, PaletteEditConfig::default());
    }

    #[test]
    fn an_unknown_key_errors() {
        let error = read(r#"{ "palette": { "edit": { "profile": {} } } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `profile`"), "{error}");
    }
}
