use crate::commands::MeshDocVoxelizeConfig;
use serde::Deserialize;

/// The `meshDoc` section of a `.vxlconfig` layer, one entry per `mesh-doc`
/// command that reads configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MeshDocConfig {
    pub(crate) voxelize: MeshDocVoxelizeConfig,
}

#[cfg(test)]
mod tests {
    use crate::commands::MeshDocConfig;
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `meshDoc` section `text` holds.
    fn read(text: &str) -> IOResult<Option<MeshDocConfig>> {
        JsoncCodec.deserialize_prefs(text.as_bytes(), "meshDoc")
    }

    #[test]
    fn an_empty_section_holds_no_command_sections() {
        let config = read(r#"{ "meshDoc": {} }"#).unwrap().unwrap();
        assert_eq!(config, MeshDocConfig::default());
    }

    #[test]
    fn a_file_without_the_section_supplies_none() {
        assert!(read(r#"{ "object": {} }"#).unwrap().is_none());
    }

    #[test]
    fn an_unknown_command_errors() {
        let error = read(r#"{ "meshDoc": { "voxelise": {} } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `voxelise`"), "{error}");
    }
}
