use crate::commands::RenderProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `object.render` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RenderConfig {
    pub(crate) profiles: BTreeMap<String, RenderProfile>,
}

#[cfg(test)]
mod tests {
    use crate::commands::{ObjectConfig, RenderConfig};
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `object.render` section `text` holds.
    fn read(text: &str) -> IOResult<RenderConfig> {
        let config: Option<ObjectConfig> =
            JsoncCodec.deserialize_prefs(text.as_bytes(), "object")?;

        Ok(config.expect("the text holds an object section").render)
    }

    #[test]
    fn the_section_reads_its_profiles_by_name() {
        let config = read(
            r##"{
  "object": {
    "render": {
      "profiles": {
        // A square.
        "square": { "width": 512, "height": 512 },
        "dusk": { "lights": [{ "kind": "hemisphere", "sky": "#402030" }] },
      },
    },
  },
}"##,
        )
        .unwrap();

        let names: Vec<_> = config.profiles.keys().collect();
        assert_eq!(names, ["dusk", "square"]);
        assert_eq!(config.profiles["square"].width.map(u32::from), Some(512));
    }

    #[test]
    fn an_unknown_key_errors() {
        let error = read(r#"{ "object": { "render": { "profile": {} } } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `profile`"), "{error}");
    }
}
