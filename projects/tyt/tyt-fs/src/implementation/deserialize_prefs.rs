use crate::Prefs;
use std::io::Result as IOResult;
use tyt_preferences::DeserializePrefs;

impl DeserializePrefs for Prefs {
    fn deserialize_prefs(config_jsonc: &[u8], key: &str) -> IOResult<Option<Self>> {
        tyt_preferences::deserialize_prefs_jsonc(config_jsonc, key)
    }
}
