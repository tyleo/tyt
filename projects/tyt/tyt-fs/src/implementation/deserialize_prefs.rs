use crate::Prefs;
use std::io::Result as IOResult;
use tyt_preferences::DeserializePrefs;

impl DeserializePrefs for Prefs {
    fn deserialize_prefs(config_json: &[u8], key: &str) -> IOResult<Option<Self>> {
        tyt_injection::parse_json_section(config_json, key)
    }
}
