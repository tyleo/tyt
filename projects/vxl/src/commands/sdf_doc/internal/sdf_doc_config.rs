use crate::commands::{SdfDocBuildConfig, SdfDocVoxelizeConfig};
use serde::Deserialize;

/// The `sdfDoc` section of a `.vxlconfig` layer, which holds an entry per
/// `sdf-doc` command that reads configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SdfDocConfig {
    pub(crate) build: SdfDocBuildConfig,

    pub(crate) voxelize: SdfDocVoxelizeConfig,
}

#[cfg(test)]
mod tests {
    use crate::commands::SdfDocConfig;
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    fn read(text: &str) -> IOResult<Option<SdfDocConfig>> {
        JsoncCodec.deserialize_prefs(text.as_bytes(), "sdfDoc")
    }

    #[test]
    fn an_empty_section_holds_no_command_sections() {
        let config = read(r#"{ "sdfDoc": {} }"#).unwrap().unwrap();
        assert_eq!(config, SdfDocConfig::default());
    }

    #[test]
    fn a_file_without_the_section_supplies_none() {
        assert!(read(r#"{ "meshDoc": {} }"#).unwrap().is_none());
    }

    #[test]
    fn an_unknown_command_errors() {
        let error = read(r#"{ "sdfDoc": { "built": {} } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `built`"), "{error}");
    }
}
