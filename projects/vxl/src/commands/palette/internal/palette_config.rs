use crate::{
    QuantizeConfig,
    commands::{PaletteEditConfig, PaletteShowConfig},
};
use serde::Deserialize;

/// The `palette` section of a `.vxlconfig` layer, one entry per `palette`
/// command that reads configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct PaletteConfig {
    pub(crate) edit: PaletteEditConfig,

    pub(crate) quantize: QuantizeConfig,

    pub(crate) show: PaletteShowConfig,
}

#[cfg(test)]
mod tests {
    use crate::commands::PaletteConfig;
    use std::io::Result as IOResult;
    use ty_preferences::{DeserializePrefs, JsoncCodec};

    /// The `palette` section `text` holds.
    fn read(text: &str) -> IOResult<Option<PaletteConfig>> {
        JsoncCodec.deserialize_prefs(text.as_bytes(), "palette")
    }

    #[test]
    fn an_empty_section_holds_no_command_sections() {
        let config = read(r#"{ "palette": {} }"#).unwrap().unwrap();
        assert_eq!(config, PaletteConfig::default());
    }

    #[test]
    fn a_file_without_the_section_supplies_none() {
        assert!(read(r#"{ "object": {} }"#).unwrap().is_none());
    }

    #[test]
    fn an_unknown_command_errors() {
        let error = read(r#"{ "palette": { "list": {} } }"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `list`"), "{error}");
    }
}
