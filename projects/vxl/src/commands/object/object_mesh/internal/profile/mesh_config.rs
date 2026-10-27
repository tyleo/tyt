use crate::commands::MeshProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `object.mesh` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MeshConfig {
    pub(crate) profiles: BTreeMap<String, MeshProfile>,
}

#[cfg(test)]
mod tests {
    use crate::commands::{MeshConfig, ObjectConfig};
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `object.mesh` section `text` holds.
    fn read(text: &str) -> IOResult<MeshConfig> {
        let config: Option<ObjectConfig> =
            JsoncCodec.deserialize_prefs(text.as_bytes(), "object")?;

        Ok(config.expect("the text holds an object section").mesh)
    }

    #[test]
    fn the_section_reads_its_profiles_by_name() {
        let config = read(
            r#"{
  "object": {
    "mesh": {
      "profiles": {
        // A mixin.
        "base": { "values": ["x = 1"] },
        "top": { "valuesFrom": ["base"], "values": ["y = x"] },
      },
    },
  },
}"#,
        )
        .unwrap();

        let names: Vec<_> = config.profiles.keys().collect();
        assert_eq!(names, ["base", "top"]);
        assert_eq!(config.profiles["top"].values_from, ["base"]);
    }

    #[test]
    fn an_empty_section_holds_no_profiles() {
        let config = read(r#"{ "object": { "mesh": {} } }"#).unwrap();
        assert_eq!(config, MeshConfig::default());
    }

    #[test]
    fn an_unknown_key_errors() {
        let error = read(r#"{ "object": { "mesh": { "profile": {} } } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `profile`"), "{error}");
    }
}
