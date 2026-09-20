use crate::commands::Profile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `mesh` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct MeshConfig {
    pub(crate) profiles: BTreeMap<String, Profile>,
}

#[cfg(test)]
mod tests {
    use super::MeshConfig;
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `mesh` section `text` holds.
    fn read(text: &str) -> IOResult<Option<MeshConfig>> {
        JsoncCodec.deserialize_prefs(text.as_bytes(), "mesh")
    }

    #[test]
    fn the_section_reads_its_profiles_by_name() {
        let config = read(
            r#"{
  "mesh": {
    "profiles": {
      // A mixin.
      "base": { "values": ["x = 1"] },
      "top": { "valuesFrom": ["base"], "values": ["y = x"] },
    },
  },
}"#,
        )
        .unwrap()
        .unwrap();

        let names: Vec<_> = config.profiles.keys().collect();
        assert_eq!(names, ["base", "top"]);
        assert_eq!(config.profiles["top"].values_from, ["base"]);
    }

    #[test]
    fn an_empty_section_holds_no_profiles() {
        let config = read(r#"{ "mesh": {} }"#).unwrap().unwrap();
        assert_eq!(config, MeshConfig::default());
    }

    #[test]
    fn a_file_without_the_section_supplies_none() {
        assert!(read(r#"{ "other": {} }"#).unwrap().is_none());
    }

    #[test]
    fn an_unknown_key_errors() {
        let error = read(r#"{ "mesh": { "profile": {} } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `profile`"), "{error}");
    }
}
