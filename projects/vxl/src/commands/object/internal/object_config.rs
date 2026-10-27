use crate::commands::{MeshConfig, ObjectVoxelsConfig, RenderConfig};
use serde::Deserialize;

/// The `object` section of a `.vxlconfig` layer, one entry per `object`
/// command that reads configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ObjectConfig {
    pub(crate) mesh: MeshConfig,

    pub(crate) render: RenderConfig,

    pub(crate) voxels: ObjectVoxelsConfig,
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectConfig;
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `object` section `text` holds.
    fn read(text: &str) -> IOResult<Option<ObjectConfig>> {
        JsoncCodec.deserialize_prefs(text.as_bytes(), "object")
    }

    #[test]
    fn an_empty_section_holds_no_command_sections() {
        let config = read(r#"{ "object": {} }"#).unwrap().unwrap();
        assert_eq!(config, ObjectConfig::default());
    }

    #[test]
    fn a_file_without_the_section_supplies_none() {
        assert!(read(r#"{ "mesh": {} }"#).unwrap().is_none());
    }

    #[test]
    fn an_unknown_command_errors() {
        let error = read(r#"{ "object": { "meshes": {} } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `meshes`"), "{error}");
    }
}
