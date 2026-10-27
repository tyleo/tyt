use crate::commands::PaletteShowProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `palette.show` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct PaletteShowConfig {
    pub(crate) profiles: BTreeMap<String, PaletteShowProfile>,
}

#[cfg(test)]
mod tests {
    use crate::commands::{PaletteConfig, PaletteShowConfig};
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `palette.show` section `text` holds.
    fn read(text: &str) -> IOResult<PaletteShowConfig> {
        let config: Option<PaletteConfig> =
            JsoncCodec.deserialize_prefs(text.as_bytes(), "palette")?;

        Ok(config.expect("the text holds a palette section").show)
    }

    #[test]
    fn the_section_reads_its_profiles_by_name() {
        let config = read(
            r#"{
  "palette": {
    "show": {
      "profiles": {
        // A mixin.
        "pbr": { "properties": [{ "property": "baseColor" }] },
        "pbr-table": { "propertiesFrom": ["pbr"], "layout": "md-tables" },
      },
    },
  },
}"#,
        )
        .unwrap();

        let names: Vec<_> = config.profiles.keys().collect();
        assert_eq!(names, ["pbr", "pbr-table"]);
        assert_eq!(config.profiles["pbr-table"].properties_from, ["pbr"]);
    }

    #[test]
    fn an_empty_section_holds_no_profiles() {
        let config = read(r#"{ "palette": { "show": {} } }"#).unwrap();
        assert_eq!(config, PaletteShowConfig::default());
    }

    #[test]
    fn an_unknown_key_errors() {
        let error = read(r#"{ "palette": { "show": { "profile": {} } } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `profile`"), "{error}");
    }
}
